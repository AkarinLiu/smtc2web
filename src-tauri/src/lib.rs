// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use serde::Serialize;
use std::net::IpAddr;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use warp::Filter;

mod config;
mod console;
mod tray;
mod theme;

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

    if let Ok(info) = session.TryGetMediaPropertiesAsync().and_then(|f| f.get()) {
        if let Ok(thumbnail) = info.Thumbnail() {
            if let Ok(stream) = thumbnail.OpenReadAsync().and_then(|f| f.get()) {
                let size = stream.Size().ok()?;
                if size > 0 && size < 10 * 1024 * 1024 {
                    let buffer = Buffer::Create(size as u32).ok()?;
                    if let Ok(read_operation) = stream.ReadAsync(
                        &buffer,
                        size as u32,
                        windows::Storage::Streams::InputStreamOptions::ReadAhead,
                    ) {
                        if let Ok(result_buffer) = read_operation.get() {
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
            }
        }
    }
    None
}

type Shared = Arc<RwLock<Song>>;

/* ---------- 主题文件托管由 theme.rs 提供 ---------- */

fn with_state(
    s: Shared,
) -> impl Filter<Extract = (Shared,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || s.clone())
}

// 单进程检测
use named_lock::NamedLock;
use open::that;
fn check_single_instance() -> Result<NamedLock, String> {
    let lock = NamedLock::create("smtc2web_single_instance_mutex")
        .map_err(|_| "创建命名锁失败".to_string())?;

    // 检查是否能获取锁
    if let Err(_) = lock.try_lock() {
        // 锁已被其他进程占用，返回错误
        let config = config::Config::load().unwrap_or_default();
        let port = config.server_port;
        let url = format!("http://localhost:{}", port);
        if let Err(e) = that(&url) {
            eprintln!("打开浏览器失败: {}", e);
        }
        return Err("程序已在运行".to_string());
    }
    
    // 锁已释放，可以安全返回锁对象
    Ok(lock)
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
            eprintln!("Failed to get session manager");
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 单进程检测 - 确保只有一个实例运行
    let _mutex_handle = match check_single_instance() {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // 检查是否是重启后的实例
    let args: Vec<String> = std::env::args().collect();
    let is_restarted = args.contains(&"--restarted".to_string());

    if is_restarted {
        println!("应用程序已重启");
        // 等待一小段时间确保前一个实例完全退出
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

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
    let (port, raw_theme_path) = {
        let config_guard = config.lock().unwrap();
        (config_guard.server_port, config_guard.theme_path.clone())
    };

    // 自动处理 Windows 路径，将双反斜杠替换为单反斜杠
    let theme_path = raw_theme_path.replace("\\\\", "\\");

    // 创建主题管理器
    let theme_manager = theme::ThemeManager::new(&theme_path);

    // JSON 接口
    let api = warp::path!("api" / "now")
        .and(with_state(state))
        .map(|s: Shared| warp::reply::json(&*s.read().unwrap()));

    // 静态文件托管（主题文件）
    let static_files = warp::path::tail()
        .and(theme::ThemeManager::with_manager(theme_manager))
        .and_then(|tail, manager: theme::ThemeManager| manager.serve_theme_file(tail));

    println!("Server running at http://localhost:{}", port);

    // 在Tokio运行时中启动Web服务器
    let _server_handle = runtime.spawn(async move {
        let address = config.clone().lock().unwrap().address.parse::<IpAddr>().expect("Invalid IP address in config");
        warp::serve(api.or(static_files))
            .run((address, port))
            .await;
    });

    // 使用 Tauri 应用程序，配置托盘事件处理
    let port_clone = port;
    tauri::Builder::default()
        .plugin(tauri_plugin_media::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // 创建系统托盘图标并配置事件处理
            tray::create_tray_icon(app.handle(), port_clone)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
