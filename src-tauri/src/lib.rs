// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use tauri::Manager;
use tokio::sync::oneshot;
use warp::Filter;

mod config;
mod font;
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
    #[serde(skip_serializing_if = "String::is_empty")]
    pub font_family: String,
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

static CURRENT_APP_ID: std::sync::LazyLock<Mutex<String>> =
    std::sync::LazyLock::new(|| Mutex::new(String::new()));

static CURRENT_APP_DISPLAY_NAME: std::sync::LazyLock<Mutex<String>> =
    std::sync::LazyLock::new(|| Mutex::new(String::new()));

static APP_STATE: std::sync::LazyLock<Mutex<AppState>> = std::sync::LazyLock::new(|| {
    Mutex::new(AppState {
        config: Arc::new(Mutex::new(config::Config::default())),
        server_tx: None,
        server_port: 3030,
        shared_state: None,
    })
});

// Cached serialized /api/now payload so browser polls don't re-clone the Song
// (deep-copying a large base64 album-art String) nor re-serialize the JSON on
// every request. The fingerprint deliberately excludes `last_update` and only
// uses the album-art *length* (O(1)), so a position tick within a second reuses
// the cached response instead of re-rendering the whole cover each poll.
type NowCache = std::sync::LazyLock<std::sync::Mutex<Option<(String, Vec<u8>)>>>;
static NOW_CACHE: NowCache = std::sync::LazyLock::new(|| Mutex::new(None));

/* ---------- 主题文件托管由 theme.rs 提供 ---------- */

fn with_state(
    s: Shared,
) -> impl Filter<Extract = (Shared,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || s.clone())
}

// Cheap content fingerprint — small strings plus the album-art *length*, never
// touching (or copying) the big base64 payload itself.
fn now_fingerprint(song: &Song, font_family: &str) -> String {
    let mut fp = String::new();
    fp.push_str(&song.title);
    fp.push('\u{1}');
    fp.push_str(&song.artist);
    fp.push('\u{1}');
    fp.push_str(&song.album);
    fp.push('\u{1}');
    fp.push_str(if song.is_playing { "1" } else { "0" });
    fp.push('\u{1}');
    if let Some(p) = &song.position {
        fp.push_str(p);
    }
    fp.push('\u{1}');
    if let Some(d) = &song.duration {
        fp.push_str(d);
    }
    fp.push('\u{1}');
    if let Some(pct) = song.pct {
        fp.push_str(&format!("{}", pct));
    }
    fp.push('\u{1}');
    fp.push_str(font_family);
    fp.push('\u{1}');
    if let Some(art) = &song.album_art {
        fp.push_str("art-len:");
        fp.push_str(&art.len().to_string());
    }
    fp
}

// Serve /api/now from a cached serialized payload. On a cache hit we only memcpy
// the already-serialized bytes and wrap them into a response; the full Song
// clone + JSON (re)serialization happens at most once per content change, not
// once per browser poll (the default theme polls every 100–200 ms).
fn handle_now(s: &Shared) -> warp::reply::Response {
    let mut cache = NOW_CACHE.lock().unwrap();
    let song_guard = s.read().unwrap();

    let font_family = APP_STATE
        .lock()
        .ok()
        .and_then(|app| app.config.lock().ok().map(|c| c.font_family.clone()))
        .unwrap_or_default();

    let fp = now_fingerprint(&song_guard, &font_family);
    let hit = match &*cache {
        Some((cached_fp, bytes)) if cached_fp == &fp => Some(bytes.clone()),
        _ => None,
    };

    let body: Vec<u8> = match hit {
        Some(bytes) => bytes,
        None => {
            let mut song = song_guard.clone();
            song.font_family = font_family;
            let bytes = serde_json::to_vec(&song).unwrap_or_default();
            *cache = Some((fp, bytes.clone()));
            bytes
        }
    };

    let mut resp = warp::reply::Response::new(warp::hyper::Body::from(body));
    resp.headers_mut().insert(
        "content-type",
        "application/json".parse().expect("content-type header"),
    );
    resp
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

        // SAFETY: CreateMutexW expects a valid null-terminated UTF-16 name (we provide one).
        // The returned HANDLE is checked for validity and error code 183 (ERROR_ALREADY_EXISTS)
        // is handled. GetLastError must be called immediately after CreateMutexW per MSDN.
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
        // SAFETY: self.handle is a valid HANDLE from CreateMutexW, not yet closed (we own it).
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

#[cfg(target_os = "linux")]
pub struct SingleInstance {
    _guard: named_lock::NamedLockGuard,
}

#[cfg(target_os = "linux")]
impl SingleInstance {
    fn new(name: &str) -> Result<Self, String> {
        use named_lock::NamedLock;

        let lock: &'static NamedLock = Box::leak(Box::new(
            NamedLock::create(name).map_err(|e| format!("创建互斥锁失败: {}", e))?,
        ));
        match lock.try_lock() {
            Ok(guard) => Ok(SingleInstance { _guard: guard }),
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

// -------------------- 后台媒体事件 (Windows: 事件驱动, Linux: 轮询) --------------------
fn media_worker(state: Shared) {
    let process_filter = {
        let app_state = APP_STATE.lock().unwrap();
        let config = app_state.config.lock().unwrap();
        config.process_filter.clone()
    };

    #[cfg(target_os = "windows")]
    {
        if let Err(e) = media::smtc::run_event_driven(state, &process_filter) {
            log_error!("Media event worker failed: {}", e);
        }
    }

    #[cfg(target_os = "linux")]
    {
        use media::{MediaSession, PlatformSession};

        let session = match PlatformSession::new(&process_filter) {
            Ok(s) => s,
            Err(e) => {
                log_error!("Failed to create media session: {}", e);
                return;
            }
        };

        media::poll_media_loop(&session, &state, &CURRENT_APP_ID, &CURRENT_APP_DISPLAY_NAME);
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
        .and(with_state(state.clone()))
        .map(|s: Shared| handle_now(&s));

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

    // 同时清理 git_themes 记录
    {
        let app_state = APP_STATE.lock().map_err(|e| e.to_string())?;
        let mut config = app_state.config.lock().map_err(|e| e.to_string())?;
        config.git_themes.remove(&theme_folder);
        let _ = config.save();
    }

    theme_manager::ThemeManager::delete_theme(&theme_folder).map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_theme_from_git(repo_url: String, branch: String) -> Result<String, String> {
    let folder_name = theme_manager::ThemeManager::install_from_git(&repo_url, &branch)
        .map_err(|e| e.to_string())?;

    // 保存到配置
    let app_state = APP_STATE.lock().map_err(|e| e.to_string())?;
    let mut config = app_state.config.lock().map_err(|e| e.to_string())?;
    config.git_themes.insert(
        folder_name.clone(),
        config::GitThemeInfo {
            repo_url,
            branch,
            folder_name: folder_name.clone(),
        },
    );
    config.save().map_err(|e| e.to_string())?;

    Ok(folder_name)
}

#[tauri::command]
async fn update_git_theme(folder_name: String) -> Result<(), String> {
    theme_manager::ThemeManager::update_git_theme(&folder_name).map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_git_theme_update(folder_name: String) -> Result<bool, String> {
    theme_manager::ThemeManager::check_git_update(&folder_name).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_git_themes() -> Result<std::collections::HashMap<String, config::GitThemeInfo>, String>
{
    let app_state = APP_STATE.lock().map_err(|e| e.to_string())?;
    let config = app_state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.git_themes.clone())
}

#[derive(Serialize, Deserialize)]
struct ConfigDto {
    server_port: u16,
    address: String,
    current_theme: String,
    locale: String,
    process_filter: String,
    update_source: String,
    auto_check_update: bool,
    minimize_to_tray: bool,
    font_family: String,
}

#[tauri::command]
async fn get_config() -> Result<ConfigDto, String> {
    let app_state = APP_STATE.lock().map_err(|e| e.to_string())?;
    let config = app_state.config.lock().map_err(|e| e.to_string())?;
    Ok(ConfigDto {
        server_port: config.server_port,
        address: config.address.clone(),
        current_theme: config.current_theme.clone(),
        locale: config.locale.clone(),
        process_filter: config.process_filter.clone(),
        update_source: config.update_source.clone(),
        auto_check_update: config.auto_check_update,
        minimize_to_tray: config.minimize_to_tray,
        font_family: config.font_family.clone(),
    })
}

#[tauri::command]
async fn save_config(config_dto: ConfigDto) -> Result<(), String> {
    let app_state = APP_STATE.lock().map_err(|e| e.to_string())?;
    let mut config = app_state.config.lock().map_err(|e| e.to_string())?;

    config.server_port = config_dto.server_port;
    config.address = config_dto.address;
    config.current_theme = config_dto.current_theme;
    config.locale = config_dto.locale;
    config.process_filter = config_dto.process_filter;
    config.update_source = config_dto.update_source;
    config.auto_check_update = config_dto.auto_check_update;
    config.minimize_to_tray = config_dto.minimize_to_tray;
    config.font_family = config_dto.font_family;

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

/// 清理历史遗留的开机自启动注册表项（开机自启动功能已移除）
fn cleanup_autostart_registry() {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Registry::{
            HKEY_CURRENT_USER, KEY_WRITE, REG_OPTION_NON_VOLATILE, RegCloseKey, RegCreateKeyExW,
            RegDeleteValueW,
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
                return;
            }
            // 删除历史遗留的 smtc2web 自启动值
            let _ = RegDeleteValueW(hkey, value_name);
            let _ = RegCloseKey(hkey);
        }
    }
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

#[tauri::command]
async fn list_system_fonts() -> Result<Vec<String>, String> {
    Ok(font::list_system_fonts())
}

/// Windows 11 圆角适配：通过 DWM API 为无边框窗口启用原生圆角
#[cfg(target_os = "windows")]
fn apply_window_rounded_corners(window: &tauri::WebviewWindow) {
    // ponytail: raw FFI to avoid windows crate version conflict with Tauri's bundled windows crate
    unsafe extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: isize,
            dw_attribute: u32,
            pv_attribute: *const std::ffi::c_void,
            cb_attribute: u32,
        ) -> i32;
    }

    if let Ok(hwnd) = window.hwnd() {
        // DWMWA_WINDOW_CORNER_PREFERENCE = 33
        // DWMWCP_ROUND = 2
        let corner: u32 = 2;
        unsafe {
            DwmSetWindowAttribute(
                hwnd.0 as isize,
                33,
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

    // 清理历史遗留的开机自启动注册表项（该功能已移除）
    cleanup_autostart_registry();

    let port_clone = port;
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_themes,
            get_current_theme,
            set_theme,
            upload_theme,
            upload_theme_from_bytes,
            delete_theme,
            install_theme_from_git,
            update_git_theme,
            check_git_theme_update,
            get_git_themes,
            get_config,
            save_config,
            set_locale,
            get_current_app_id,
            updater::check_update,
            updater::start_update,
            window_minimize,
            window_toggle_maximize,
            window_close,
            window_is_maximized,
            open_url,
            list_system_fonts
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

            // 标题栏：所有平台统一使用系统原生标题栏（tauri.conf.json 已配置
            // decorations: true），此处不再做任何运行时装饰调整。

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

            // 启动时最小化至托盘（由配置 minimize_to_tray 控制）
            let minimize_to_tray = {
                let app_state = APP_STATE.lock().unwrap();
                let config_guard = app_state.config.lock().unwrap();
                config_guard.minimize_to_tray
            };
            if minimize_to_tray {
                log_info!("启动时最小化至系统托盘");
                let _ = window.hide();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    runtime.block_on(async {
        let _ = server_handle.await;
    });
}
