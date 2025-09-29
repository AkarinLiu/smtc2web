// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use rust_embed::RustEmbed;
use serde::Serialize;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use warp::{path::Tail, Filter};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONERROR};

mod config;
mod console;
mod tray;

#[derive(RustEmbed)]
#[folder = "frontend"]
struct Asset;

#[derive(Default, Clone, Serialize, PartialEq)]
struct Song {
    title: String,
    artist: String,
    album: String,
    position: Option<String>,
    duration: Option<String>,
    pct: Option<f64>, // 修改为f64类型以支持小数
    is_playing: bool,
    last_update: u64, // 时间戳用于强制更新
}

// 将秒数格式化为 MM:SS 格式
fn format_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", minutes, secs)
}

type Shared = Arc<RwLock<Song>>;

/* ---------- 内存静态文件托管 ---------- */
async fn serve_embed(tail: Tail) -> Result<impl warp::Reply, warp::Rejection> {
    let path = tail.as_str();
    let path = if path.is_empty() { "index.html" } else { path };
    match Asset::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Ok(warp::reply::with_header(
                content.data.to_vec(),
                "content-type",
                mime.as_ref(),
            ))
        },
        None => Err(warp::reject::not_found()),
    }
}

fn with_state(
    s: Shared,
) -> impl Filter<Extract = (Shared,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || s.clone())
}

// -------------------- 单进程检测 --------------------
fn check_single_instance() -> Result<HANDLE, String> {
    unsafe {
        let mutex_name = "smtc2web_single_instance_mutex";
        let mutex_name_wide: Vec<u16> = mutex_name.encode_utf16().chain(Some(0)).collect();
        
        let handle = CreateMutexW(None, true, windows::core::PCWSTR::from_raw(mutex_name_wide.as_ptr()));
        
        match handle {
            Ok(h) => {
                // 检查错误码来判断是否已存在
                use windows::Win32::Foundation::ERROR_ALREADY_EXISTS;
                use windows::Win32::Foundation::GetLastError;
                
                let error_code = GetLastError();
                
                if error_code == ERROR_ALREADY_EXISTS {
                    // 显示错误消息框
                    let title = "smtc2web";
                    let message = "程序已经在运行中，请勿重复启动！";
                    let title_wide: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
                    let message_wide: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
                    
                    MessageBoxW(
                        None,
                        windows::core::PCWSTR::from_raw(message_wide.as_ptr()),
                        windows::core::PCWSTR::from_raw(title_wide.as_ptr()),
                        MB_OK | MB_ICONERROR
                    );
                    
                    return Err("程序已在运行".to_string());
                }
                
                Ok(h)
            },
            Err(_) => Err("创建互斥量失败".to_string())
        }
    }
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
        },
    };

    let mut last_song = Song::default();
    let mut last_position = None::<String>;
    let mut no_change_count = 0;

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

        // 智能更新逻辑
        let should_update = if current_song.is_playing != last_song.is_playing {
            // 播放状态变化，立即更新
            true
        } else if current_song.position != last_position {
            // 位置变化，更新
            true
        } else if !current_song.is_playing && no_change_count < 10 {
            // 暂停状态下，前几次继续更新以确保状态同步
            no_change_count += 1;
            true
        } else {
            // 其他情况，检查是否需要强制更新
            timestamp.saturating_sub(last_song.last_update) > 5 // 5秒强制更新一次
        };

        if should_update {
            let mut s = state.write().unwrap();
            *s = current_song.clone();
            last_song = current_song.clone();
            last_position = current_song.position;
            no_change_count = 0;
        }

        // 动态调整轮询间隔
        let sleep_duration = if current_song.is_playing {
            Duration::from_millis(100) // 播放时更频繁更新
        } else {
            Duration::from_millis(200) // 暂停时减少频率
        };

        std::thread::sleep(sleep_duration);
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 单进程检测 - 确保只有一个实例运行
    let _mutex_handle = match check_single_instance() {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        },
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
    let config = Arc::new(Mutex::new(config::Config::load()));

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

    // 获取服务器端口
    let port = {
        let config_guard = config.lock().unwrap();
        config_guard.server_port
    };

    // JSON 接口
    let api = warp::path!("api" / "now")
        .and(with_state(state))
        .map(|s: Shared| warp::reply::json(&*s.read().unwrap()));

    // 静态文件托管（内存 embed）
    let static_files = warp::path::tail().and_then(serve_embed);

    println!("Server running at http://localhost:{}", port);

    // 在Tokio运行时中启动Web服务器
    let _server_handle = runtime.spawn(async move {
        warp::serve(api.or(static_files))
            .run(([127, 0, 0, 1], port))
            .await;
    });

    // 使用 Tauri 应用程序，配置托盘事件处理
    let port_clone = port;
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(move |app| {
            // 创建系统托盘图标并配置事件处理
            tray::create_tray_icon(app.handle(), port_clone)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");


}
