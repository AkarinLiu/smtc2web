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
mod logger;
mod theme;
mod theme_manager;
mod tray;

#[derive(Default, Clone, Serialize, PartialEq)]
struct Song {
    title: String,
    artist: String,
    album: String,
    album_art: Option<String>,
    position: Option<String>,
    duration: Option<String>,
    pct: Option<f64>,
    is_playing: bool,
    last_update: u64,
}

// 将秒数格式化为 MM:SS 格式
fn format_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", minutes, secs)
}

async fn get_album_art(
    session: &windows::Media::Control::GlobalSystemMediaTransportControlsSession,
) -> Option<String> {
    use windows::Storage::Streams::Buffer;
    use windows::Storage::Streams::DataReader;

    if let Ok(info) = session.TryGetMediaPropertiesAsync().and_then(|f| f.get())
        && let Ok(thumbnail) = info.Thumbnail()
        && let Ok(stream) = thumbnail.OpenReadAsync().and_then(|f| f.get())
    {
        let size = stream.Size().ok()?;
        if size > 0 && size < 10 * 1024 * 1024 {
            let buffer = Buffer::Create(size as u32).ok()?;
            if let Ok(read_operation) = stream.ReadAsync(
                &buffer,
                size as u32,
                windows::Storage::Streams::InputStreamOptions::ReadAhead,
            ) && let Ok(result_buffer) = read_operation.get()
            {
                let reader = DataReader::FromBuffer(&result_buffer).ok()?;
                let length = result_buffer.Length().ok()? as usize;
                let mut data = vec![0u8; length];
                reader.ReadBytes(&mut data).ok()?;
                use base64::{Engine, engine::general_purpose::STANDARD};
                let mime = "data:image/jpeg";
                let data_uri = format!("{};base64,{}", mime, STANDARD.encode(&data));
                return Some(data_uri);
            }
        }
    }
    None
}

type Shared = Arc<RwLock<Song>>;

// 全局状态
struct AppState {
    config: Arc<Mutex<config::Config>>,
    server_tx: Option<oneshot::Sender<()>>,
    server_port: u16,
    shared_state: Option<Shared>,
}

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

// 单进程检测 - 使用 Windows CreateMutex
use open::that;
use std::ptr;
use windows::Win32::Foundation::{CloseHandle, HANDLE, WIN32_ERROR};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::Foundation::GetLastError;

pub struct SingleInstance {
    handle: HANDLE,
}

impl SingleInstance {
    pub fn new(name: &str) -> Result<Self, String> {
        // 将 Rust 字符串转换为宽字符
        let name_wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        
        unsafe {
            let handle_result = CreateMutexW(
                Some(ptr::null()),  // 安全属性
                false,              // 初始不持有
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
            // ERROR_ALREADY_EXISTS 表示互斥量已存在，说明已有实例在运行
            if error == WIN32_ERROR(183) {
                // 183 = ERROR_ALREADY_EXISTS
                let _ = CloseHandle(handle);
                
                // 打开浏览器
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

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

fn check_single_instance() -> Result<SingleInstance, String> {
    SingleInstance::new("smtc2web_single_instance_mutex")
}

// -------------------- 后台轮询 --------------------
fn smtc_worker(state: Shared) {
    use std::time::{SystemTime, UNIX_EPOCH};
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

    let manager = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .and_then(|f| f.get())
    {
        Ok(m) => m,
        Err(_) => {
            log_error!("Failed to get session manager");
            return;
        }
    };

    let mut last_song = Song::default();
    let mut last_position = None::<String>;

    loop {
        let mut current_song = Song::default();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        current_song.last_update = timestamp;

        if let Ok(session) = manager.GetCurrentSession() {
            // 获取播放状态
            if let Ok(playback_info) = session.GetPlaybackInfo() {
                use windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus;
                current_song.is_playing = playback_info.PlaybackStatus().unwrap_or_default()
                    == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
            }

            // 元数据
            if let Ok(info) = session.TryGetMediaPropertiesAsync().and_then(|f| f.get()) {
                current_song.title = info.Title().unwrap_or_default().to_string();
                current_song.artist = info.Artist().unwrap_or_default().to_string();
                current_song.album = info.AlbumTitle().unwrap_or_default().to_string();
            }

            // 专辑图片
            let runtime = tokio::runtime::Runtime::new().unwrap();
            current_song.album_art = runtime.block_on(get_album_art(&session));

            // 进度
            if let Ok(timeline) = session.GetTimelineProperties() {
                let pos = timeline.Position().unwrap().Duration;
                let dur = timeline.EndTime().unwrap().Duration;
                let pos_s = pos / 10_000_000;
                let dur_s = dur / 10_000_000;

                if dur != 0 {
                    current_song.position = Some(format_duration(pos_s as u64));
                    current_song.duration = Some(format_duration(dur_s as u64));
                    let percentage = (pos_s as f64 * 100.0) / dur_s as f64;
                    current_song.pct = Some((percentage * 10.0).round() / 10.0);
                } else {
                    current_song.position = None;
                    current_song.duration = None;
                    current_song.pct = None;
                }
            }
        }

        // 简化的更新逻辑
        let should_update =
            // 播放状态变化
            current_song.is_playing != last_song.is_playing ||
            // 位置变化
            current_song.position != last_position ||
            // 元数据变化
            current_song.title != last_song.title ||
            current_song.artist != last_song.artist ||
            current_song.album != last_song.album ||
            // 专辑图片变化
            current_song.album_art != last_song.album_art ||
            // 强制更新（每5秒）
            timestamp.saturating_sub(last_song.last_update) > 5;

        if should_update {
            let mut s = state.write().unwrap();
            *s = current_song.clone();
            last_song = current_song.clone();
            last_position = current_song.position;
        }

        // 根据播放状态调整轮询间隔
        let sleep_duration = match current_song.is_playing {
            true => Duration::from_millis(100),
            false => Duration::from_millis(200),
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
    // 获取服务器地址
    let address = {
        let app_state = APP_STATE.lock().unwrap();
        let config = app_state.config.lock().unwrap();
        config
            .address
            .parse::<IpAddr>()
            .expect("Invalid IP address in config")
    };

    // 确定主题路径
    // "default" 或空字符串都表示使用内置的 RustEmbed 主题
    let theme_path = if current_theme.is_empty() || current_theme == "default" {
        PathBuf::new() // 使用内置主题
    } else {
        theme_manager::ThemeManager::get_theme_server_path(&current_theme)
    };

    // 创建主题管理器
    let theme_manager = theme::ThemeManager::new(&theme_path.to_string_lossy());

    // JSON 接口
    let api = warp::path!("api" / "now")
        .and(with_state(state))
        .map(|s: Shared| warp::reply::json(&*s.read().unwrap()));

    // 主题文件托管
    let theme_files = warp::path("theme")
        .and(warp::path::tail())
        .and(theme::ThemeManager::with_manager(theme_manager.clone()))
        .and_then(|tail, manager: theme::ThemeManager| manager.serve_theme_file(tail));

    // 静态文件托管（前端）
    let static_files = warp::path::tail()
        .and(theme::ThemeManager::with_manager(theme_manager))
        .and_then(|tail, manager: theme::ThemeManager| manager.serve_theme_file(tail));

    // 创建关闭信号
    let (tx, rx) = oneshot::channel::<()>();

    // 启动服务器
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
    // 停止现有服务器并获取共享状态
    let (port, state) = {
        let mut app_state = APP_STATE.lock().map_err(|e| e.to_string())?;
        if let Some(tx) = app_state.server_tx.take() {
            let _ = tx.send(());
        }

        // 更新配置
        {
            let mut config = app_state.config.lock().map_err(|e| e.to_string())?;
            config.current_theme = theme_name.clone();
            config.save().map_err(|e| e.to_string())?;
        }

        // 获取端口和共享状态
        let port = app_state.config.lock().map_err(|e| e.to_string())?.server_port;
        let state = app_state.shared_state.clone().ok_or("Shared state not initialized")?;
        (port, state)
    };

    // 重新启动服务器，使用现有的共享状态
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

    // 创建临时文件
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(&file_name);

    // 写入文件数据
    let mut temp_file =
        std::fs::File::create(&temp_file_path).map_err(|e| format!("创建临时文件失败: {}", e))?;
    temp_file
        .write_all(&file_data)
        .map_err(|e| format!("写入临时文件失败: {}", e))?;

    // 解压主题
    let theme_name = theme_manager::ThemeManager::extract_theme(&temp_file_path)
        .map_err(|e| format!("解压主题失败: {}", e))?;

    // 删除临时文件
    let _ = std::fs::remove_file(&temp_file_path);

    Ok(theme_name)
}

#[tauri::command]
async fn delete_theme(theme_folder: String) -> Result<(), String> {
    // 检查是否是当前主题
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

    config.save().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统
    logger::init();
    log_info!("应用程序启动");

    // 单进程检测 - 确保只有一个实例运行
    // SingleInstance 必须保持在作用域内，否则锁会被释放
    let _single_instance = match check_single_instance() {
        Ok(instance) => instance,
        Err(e) => {
            log_error!("{}", e);
            std::process::exit(1);
        }
    };

    // 检查是否是重启后的实例
    let args: Vec<String> = std::env::args().collect();
    let is_restarted = args.contains(&"--restarted".to_string());

    if is_restarted {
        log_info!("应用程序已重启");
        // 等待一小段时间确保前一个实例完全退出
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // 确保主题目录存在
    theme_manager::ThemeManager::ensure_themes_dir().expect("Failed to create themes directory");

    // 加载配置
    let config = Arc::new(Mutex::new(config::Config::load().unwrap_or_default()));

    // 启动配置文件监控
    config::Config::start_monitoring(config.clone());

    // 根据配置决定是否隐藏控制台
    {
        let config_guard = config.lock().unwrap();
        if !config_guard.show_console {
            console::hide_console();
        }
    }

    // 创建 Tokio 运行时
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    // 启动 Web 服务器
    let state: Shared = Arc::default();
    let st = state.clone();

    // 在单独的线程中运行后台轮询
    std::thread::spawn(move || smtc_worker(st));

    // 获取服务器配置
    let (port, current_theme) = {
        let config_guard = config.lock().unwrap();
        (config_guard.server_port, config_guard.current_theme.clone())
    };

    // 在Tokio运行时中启动Web服务器
    let state_for_server = state.clone();
    let (server_tx, server_handle) =
        runtime.block_on(async { start_server(state_for_server, port, current_theme).await });

    // 更新全局状态
    {
        let mut app_state = APP_STATE.lock().unwrap();
        app_state.config = config.clone();
        app_state.server_tx = Some(server_tx);
        app_state.server_port = port;
        app_state.shared_state = Some(state);
    }

    // 使用 Tauri 应用程序，配置托盘事件处理
    let port_clone = port;
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_media::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_themes,
            get_current_theme,
            set_theme,
            upload_theme,
            upload_theme_from_bytes,
            delete_theme,
            get_config,
            save_config
        ])
        .setup(move |app| {
            // 创建系统托盘图标并配置事件处理
            tray::create_tray_icon(app.handle(), port_clone)?;

            // 配置窗口关闭行为：隐藏到托盘而不是退出
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    // 阻止窗口关闭
                    api.prevent_close();
                    // 隐藏窗口
                    let _ = window_clone.hide();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // 清理：停止服务器
    runtime.block_on(async {
        let _ = server_handle.await;
    });
}
