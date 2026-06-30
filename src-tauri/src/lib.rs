// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tauri::Manager;
use tokio::sync::oneshot;
use warp::Filter;

mod config;
mod console;
mod i18n;
mod logger;
mod media;
mod theme;
mod theme_manager;
mod tray;
mod updater;

pub mod cli;
pub mod dev;

#[derive(Default, Clone, Serialize, PartialEq)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_art: Option<String>,
    pub position: Option<String>,
    pub duration: Option<String>,
    pub pct: Option<f64>,
    pub is_playing: bool,
    pub last_update: u64,
}

pub fn format_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", minutes, secs)
}

pub type Shared = Arc<RwLock<Song>>;

struct AppState {
    config: Arc<Mutex<config::Config>>,
    server_tx: Option<oneshot::Sender<()>>,
    server_port: u16,
    shared_state: Option<Shared>,
}

static CURRENT_APP_ID: once_cell::sync::Lazy<Mutex<String>> =
    once_cell::sync::Lazy::new(|| Mutex::new(String::new()));

static CURRENT_APP_DISPLAY_NAME: once_cell::sync::Lazy<Mutex<String>> =
    once_cell::sync::Lazy::new(|| Mutex::new(String::new()));

static APP_STATE: once_cell::sync::Lazy<Mutex<AppState>> = once_cell::sync::Lazy::new(|| {
    Mutex::new(AppState {
        config: Arc::new(Mutex::new(config::Config::default())),
        server_tx: None,
        server_port: 3030,
        shared_state: None,
    })
});

/* ---------- 主题文件托管由 theme.rs 提供 ---------- */

fn with_state(
    s: Shared,
) -> impl Filter<Extract = (Shared,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || s.clone())
}

// 单进程检测 - 跨平台实现
use open::that;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::GetLastError;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{CloseHandle, HANDLE, WIN32_ERROR};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::CreateMutexW;

#[cfg(target_os = "windows")]
pub struct SingleInstance {
    handle: HANDLE,
}

#[cfg(target_os = "windows")]
impl SingleInstance {
    fn new(name: &str) -> Result<Self, String> {
        let name_wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let handle_result = CreateMutexW(
                Some(std::ptr::null()),
                false,
                windows::core::PCWSTR(name_wide.as_ptr()),
            );

            let handle = match handle_result {
                Ok(h) => h,
                Err(_) => return Err("创建互斥锁失败".to_string()),
            };

            if handle.is_invalid() {
                return Err("创建互斥锁失败".to_string());
            }

            let error = GetLastError();
            if error == WIN32_ERROR(183) {
                let _ = CloseHandle(handle);

                let config = config::Config::load().unwrap_or_default();
                let port = config.server_port;
                let url = format!("http://localhost:{}", port);
                if let Err(e) = that(&url) {
                    log_error!("打开浏览器失败: {}", e);
                }

                return Err("程序已在运行".to_string());
            }

            Ok(SingleInstance { handle })
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

#[cfg(target_os = "linux")]
pub struct SingleInstance {
    _lock: Box<named_lock::NamedLock>,
    _guard: named_lock::NamedLockGuard<'static>,
}

#[cfg(target_os = "linux")]
impl SingleInstance {
    fn new(name: &str) -> Result<Self, String> {
        use named_lock::NamedLock;

        let lock = Box::new(NamedLock::create(name).map_err(|e| format!("创建互斥锁失败: {}", e))?);

        // SAFETY: lock is in a Box (stable address). We'll store it in
        // SingleInstance._lock, which lives for the entire process lifetime.
        // The reference is therefore valid for 'static.
        let lock_ref: &'static NamedLock = unsafe { std::mem::transmute(lock.as_ref()) };

        match lock_ref.try_lock() {
            Ok(guard) => {
                // SAFETY: guard borrows lock_ref which has 'static lifetime.
                let guard: named_lock::NamedLockGuard<'static> =
                    unsafe { std::mem::transmute(guard) };
                Ok(SingleInstance {
                    _lock: lock,
                    _guard: guard,
                })
            }
            Err(_) => {
                let config = config::Config::load().unwrap_or_default();
                let port = config.server_port;
                let url = format!("http://localhost:{}", port);
                if let Err(e) = that(&url) {
                    log_error!("打开浏览器失败: {}", e);
                }

                Err("程序已在运行".to_string())
            }
        }
    }
}

fn check_single_instance() -> Result<SingleInstance, String> {
    SingleInstance::new("smtc2web_single_instance_mutex")
}

// -------------------- 后台轮询 --------------------
fn media_worker(state: Shared) {
    use media::{MediaSession, PlatformSession};
    use std::time::{SystemTime, UNIX_EPOCH};

    let process_filter = {
        let app_state = APP_STATE.lock().unwrap();
        let config = app_state.config.lock().unwrap();
        config.process_filter.clone()
    };

    let session = match PlatformSession::new(&process_filter) {
        Ok(s) => s,
        Err(e) => {
            log_error!("Failed to create media session: {}", e);
            return;
        }
    };

    let mut last_song = Song::default();
    let mut last_position = None::<String>;
    let mut last_song_id = String::new();
    let mut last_art_update = 0u64;

    loop {
        let mut current_song = Song::default();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        current_song.last_update = timestamp;

        if let Some(info) = session.poll_current() {
            {
                let mut current_app = CURRENT_APP_ID.lock().unwrap();
                *current_app = info.app_id.clone();
            }
            {
                let mut current_display_name = CURRENT_APP_DISPLAY_NAME.lock().unwrap();
                *current_display_name = info.app_name.clone();
            }

            current_song.is_playing = info.is_playing;
            current_song.title = info.title;
            current_song.artist = info.artist;
            current_song.album = info.album;

            let current_song_id = media::generate_song_id(
                &current_song.title,
                &current_song.artist,
                &current_song.album,
            );

            let cached_art = media::get_cached_album_art(&current_song_id);

            let should_fetch_art = cached_art.is_none()
                || (current_song_id != last_song_id
                    && timestamp.saturating_sub(last_art_update) > 30);

            if should_fetch_art {
                current_song.album_art = session.get_album_art_base64(
                    &current_song.artist,
                    &current_song.title,
                    &current_song.album,
                );
                last_song_id = current_song_id;
                last_art_update = timestamp;
            } else {
                current_song.album_art = cached_art;
            }

            if info.duration_secs > 0 {
                current_song.position = Some(format_duration(info.position_secs));
                current_song.duration = Some(format_duration(info.duration_secs));
                let percentage = (info.position_secs as f64 * 100.0) / info.duration_secs as f64;
                current_song.pct = Some((percentage * 10.0).round() / 10.0);
            }
        } else {
            let empty_song = Song::default();
            let mut s = state.write().unwrap();
            *s = empty_song.clone();
            last_song = empty_song.clone();
            last_position = None;

            std::thread::sleep(Duration::from_millis(500));
            continue;
        }

        let should_update = current_song.is_playing != last_song.is_playing
            || current_song.position != last_position
            || current_song.title != last_song.title
            || current_song.artist != last_song.artist
            || current_song.album != last_song.album
            || current_song.album_art != last_song.album_art
            || timestamp.saturating_sub(last_song.last_update) > 10;

        if should_update {
            let mut s = state.write().unwrap();
            *s = current_song.clone();
            last_song = current_song.clone();
            last_position = current_song.position.clone();
        }

        let sleep_duration = match current_song.is_playing {
            true => Duration::from_millis(200),
            false => Duration::from_millis(1000),
        };
        std::thread::sleep(sleep_duration);
    }
}

// 启动 Web 服务器
async fn start_server(
    state: Shared,
    port: u16,
    current_theme: String,
) -> (oneshot::Sender<()>, tokio::task::JoinHandle<()>) {
    let address = {
        let app_state = APP_STATE.lock().unwrap();
        let config = app_state.config.lock().unwrap();
        config
            .address
            .parse::<IpAddr>()
            .expect("Invalid IP address in config")
    };

    let theme_path = if current_theme.is_empty() || current_theme == "default" {
        PathBuf::new()
    } else {
        theme_manager::ThemeManager::get_theme_server_path(&current_theme)
    };

    let theme_manager = theme::ThemeManager::new(&theme_path.to_string_lossy());

    let api = warp::path!("api" / "now")
        .and(with_state(state))
        .map(|s: Shared| warp::reply::json(&*s.read().unwrap()));

    let theme_files = warp::path("theme")
        .and(warp::path::tail())
        .and(theme::ThemeManager::with_manager(theme_manager.clone()))
        .and_then(|tail, manager: theme::ThemeManager| manager.serve_theme_file(tail));

    let static_files = warp::path::tail()
        .and(theme::ThemeManager::with_manager(theme_manager))
        .and_then(|tail, manager: theme::ThemeManager| manager.serve_theme_file(tail));

    let (tx, rx) = oneshot::channel::<()>();

    let server_handle = tokio::spawn(async move {
        let (_, server) = warp::serve(api.or(theme_files).or(static_files))
            .bind_with_graceful_shutdown((address, port), async {
                let _ = rx.await;
            });
        server.await;
    });

    log_info!("Server running at http://{}:{}", address, port);

    (tx, server_handle)
}

// -------------------- Tauri 命令 --------------------

#[tauri::command]
async fn get_themes() -> Result<Vec<theme_manager::ThemeInfo>, String> {
    theme_manager::ThemeManager::scan_themes().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_current_theme() -> Result<String, String> {
    let app_state = APP_STATE.lock().map_err(|e| e.to_string())?;
    let config = app_state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.current_theme.clone())
}

#[tauri::command]
async fn set_theme(theme_name: String, _app_handle: tauri::AppHandle) -> Result<(), String> {
    let (port, state) = {
        let mut app_state = APP_STATE.lock().map_err(|e| e.to_string())?;
        if let Some(tx) = app_state.server_tx.take() {
            let _ = tx.send(());
        }

        {
            let mut config = app_state.config.lock().map_err(|e| e.to_string())?;
            config.current_theme = theme_name.clone();
            config.save().map_err(|e| e.to_string())?;
        }

        let port = app_state
            .config
            .lock()
            .map_err(|e| e.to_string())?
            .server_port;
        let state = app_state
            .shared_state
            .clone()
            .ok_or("Shared state not initialized")?;
        (port, state)
    };

    let (tx, _) = start_server(state, port, theme_name).await;

    {
        let mut app_state = APP_STATE.lock().map_err(|e| e.to_string())?;
        app_state.server_tx = Some(tx);
    }

    Ok(())
}

#[tauri::command]
async fn upload_theme(file_path: String) -> Result<String, String> {
    let path = std::path::Path::new(&file_path);
    theme_manager::ThemeManager::extract_theme(path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn upload_theme_from_bytes(file_name: String, file_data: Vec<u8>) -> Result<String, String> {
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(&file_name);

    let mut temp_file =
        std::fs::File::create(&temp_file_path).map_err(|e| format!("创建临时文件失败: {}", e))?;
    temp_file
        .write_all(&file_data)
        .map_err(|e| format!("写入临时文件失败: {}", e))?;

    let theme_name = theme_manager::ThemeManager::extract_theme(&temp_file_path)
        .map_err(|e| format!("解压主题失败: {}", e))?;

    let _ = std::fs::remove_file(&temp_file_path);

    Ok(theme_name)
}

#[tauri::command]
async fn delete_theme(theme_folder: String) -> Result<(), String> {
    let current = {
        let app_state = APP_STATE.lock().map_err(|e| e.to_string())?;
        let config = app_state.config.lock().map_err(|e| e.to_string())?;
        config.current_theme.clone()
    };

    if current == theme_folder {
        return Err("不能删除当前正在使用的主题".to_string());
    }

    theme_manager::ThemeManager::delete_theme(&theme_folder).map_err(|e| e.to_string())
}

#[derive(Serialize, Deserialize)]
struct ConfigDto {
    server_port: u16,
    show_console: bool,
    address: String,
    current_theme: String,
    locale: String,
    process_filter: String,
    update_source: String,
    auto_check_update: bool,
    autostart: bool,
}

#[tauri::command]
async fn get_config() -> Result<ConfigDto, String> {
    let app_state = APP_STATE.lock().map_err(|e| e.to_string())?;
    let config = app_state.config.lock().map_err(|e| e.to_string())?;
    Ok(ConfigDto {
        server_port: config.server_port,
        show_console: config.show_console,
        address: config.address.clone(),
        current_theme: config.current_theme.clone(),
        locale: config.locale.clone(),
        process_filter: config.process_filter.clone(),
        update_source: config.update_source.clone(),
        auto_check_update: config.auto_check_update,
        autostart: config.autostart,
    })
}

#[tauri::command]
async fn save_config(config_dto: ConfigDto) -> Result<(), String> {
    let app_state = APP_STATE.lock().map_err(|e| e.to_string())?;
    let mut config = app_state.config.lock().map_err(|e| e.to_string())?;

    config.server_port = config_dto.server_port;
    config.show_console = config_dto.show_console;
    config.address = config_dto.address;
    config.current_theme = config_dto.current_theme;
    config.locale = config_dto.locale;
    config.process_filter = config_dto.process_filter;
    config.update_source = config_dto.update_source;
    config.auto_check_update = config_dto.auto_check_update;
    config.autostart = config_dto.autostart;

    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_locale(locale: String, app: tauri::AppHandle) -> Result<(), String> {
    log_info!("Setting locale to: {}", locale);
    tray::update_tray_menu_language(&app, &locale)
}

#[tauri::command]
async fn get_current_app_id() -> Result<String, String> {
    let display_name = CURRENT_APP_DISPLAY_NAME.lock().map_err(|e| e.to_string())?;
    Ok(display_name.clone())
}

/// 同步开机自启动注册表项（不修改配置）
fn sync_autostart(enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Registry::{
            HKEY_CURRENT_USER, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey,
            RegCreateKeyExW, RegDeleteValueW, RegSetValueExW,
        };
        use windows::core::PCWSTR;

        let subkey = windows::core::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let value_name = windows::core::w!("smtc2web");

        unsafe {
            let mut hkey = std::mem::zeroed();
            let result = RegCreateKeyExW(
                HKEY_CURRENT_USER,
                subkey,
                0u32,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                None,
                &mut hkey,
                None,
            );
            if result.is_err() {
                return Err(format!("Failed to open registry key: {:?}", result));
            }

            if enable {
                let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
                let exe_str = exe_path.to_string_lossy();
                let exe_wide: Vec<u16> = exe_str.encode_utf16().chain(std::iter::once(0)).collect();
                let data_bytes: &[u8] =
                    std::slice::from_raw_parts(exe_wide.as_ptr() as *const u8, exe_wide.len() * 2);

                let result = RegSetValueExW(hkey, value_name, 0u32, REG_SZ, Some(data_bytes));
                if result.is_err() {
                    let _ = RegCloseKey(hkey);
                    return Err(format!("Failed to set registry value: {:?}", result));
                }
            } else {
                let _ = RegDeleteValueW(hkey, value_name);
            }

            let _ = RegCloseKey(hkey);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = enable;
        return Err("Auto-start is only supported on Windows".to_string());
    }

    Ok(())
}

#[tauri::command]
async fn set_autostart(enable: bool) -> Result<(), String> {
    sync_autostart(enable)?;

    let app_state = APP_STATE.lock().map_err(|e| e.to_string())?;
    let mut config = app_state.config.lock().map_err(|e| e.to_string())?;
    config.autostart = enable;
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
async fn window_minimize(window: tauri::WebviewWindow) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
async fn window_toggle_maximize(window: tauri::WebviewWindow) -> Result<(), String> {
    let maximized = window.is_maximized().map_err(|e| e.to_string())?;
    if maximized {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn window_close(window: tauri::WebviewWindow) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
async fn window_is_maximized(window: tauri::WebviewWindow) -> Result<bool, String> {
    window.is_maximized().map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}

/// Windows 11 圆角适配：通过 DWM API 为无边框窗口启用原生圆角
#[cfg(target_os = "windows")]
fn apply_window_rounded_corners(window: &tauri::WebviewWindow) {
    // 直接通过 FFI 调用 dwmapi.dll，避免 windows crate 版本冲突
    unsafe extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: isize,
            dw_attribute: u32,
            pv_attribute: *const std::ffi::c_void,
            cb_attribute: u32,
        ) -> i32;
    }

    if let Ok(hwnd) = window.hwnd() {
        // 从 Tauri 的 HWND 中提取原始指针值
        let raw: isize = unsafe { std::mem::transmute_copy(&hwnd) };
        // DWMWA_WINDOW_CORNER_PREFERENCE = 33
        // DWMWCP_ROUND = 2
        let corner: u32 = 2;
        unsafe {
            DwmSetWindowAttribute(
                raw,
                33, // DWMWA_WINDOW_CORNER_PREFERENCE
                &corner as *const _ as *const std::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            );
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logger::init();
    log_info!("应用程序启动");

    let _single_instance = match check_single_instance() {
        Ok(instance) => instance,
        Err(e) => {
            log_error!("{}", e);
            std::process::exit(1);
        }
    };

    let args: Vec<String> = std::env::args().collect();
    let is_restarted = args.contains(&"--restarted".to_string());

    if is_restarted {
        log_info!("应用程序已重启");
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    theme_manager::ThemeManager::ensure_themes_dir().expect("Failed to create themes directory");

    let config = Arc::new(Mutex::new(config::Config::load().unwrap_or_default()));

    config::Config::start_monitoring(config.clone());

    {
        let config_guard = config.lock().unwrap();
        if !config_guard.show_console {
            console::hide_console();
        }
    }

    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    let state: Shared = Arc::default();
    let st = state.clone();

    std::thread::spawn(move || media_worker(st));

    let (port, current_theme) = {
        let config_guard = config.lock().unwrap();
        (config_guard.server_port, config_guard.current_theme.clone())
    };

    let state_for_server = state.clone();
    let (server_tx, server_handle) =
        runtime.block_on(async { start_server(state_for_server, port, current_theme).await });

    {
        let mut app_state = APP_STATE.lock().unwrap();
        app_state.config = config.clone();
        app_state.server_tx = Some(server_tx);
        app_state.server_port = port;
        app_state.shared_state = Some(state);
    }

    // 同步开机自启动设置到注册表
    {
        let app_state = APP_STATE.lock().unwrap();
        let config = app_state.config.lock().unwrap();
        let _ = sync_autostart(config.autostart);
    }

    let port_clone = port;
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        // tauri-plugin-media removed — unused, broken on Linux
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_themes,
            get_current_theme,
            set_theme,
            upload_theme,
            upload_theme_from_bytes,
            delete_theme,
            get_config,
            save_config,
            set_locale,
            get_current_app_id,
            updater::check_update,
            updater::get_update_source_urls,
            updater::start_update,
            set_autostart,
            window_minimize,
            window_toggle_maximize,
            window_close,
            window_is_maximized,
            open_url
        ])
        .setup(move |app| {
            {
                let app_state = APP_STATE.lock().unwrap();
                let config_guard = app_state.config.lock().unwrap();
                let locale = config_guard.locale.clone();
                let _ = i18n::set_locale(&locale);
                log_info!("Applied locale from config: {}", locale);
            }

            tray::create_tray_icon(app.handle(), port_clone)?;

            let window = app.get_webview_window("main").unwrap();

            // 确保无边框窗口生效（在 Windows 上有时 config 的 decorations: false 不够）
            let _ = window.set_decorations(false);

            // Windows 11 圆角适配
            #[cfg(target_os = "windows")]
            apply_window_rounded_corners(&window);

            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window_clone.hide();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    runtime.block_on(async {
        let _ = server_handle.await;
    });
}
