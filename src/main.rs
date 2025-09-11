mod config;
mod console;

use rust_embed::RustEmbed;
use serde::Serialize;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::task;
use warp::{path::Tail, Filter};

#[derive(RustEmbed)]
#[folder = "frontend"]
struct Asset;

#[derive(Default, Clone, Serialize, PartialEq)]
struct Song {
    title: String,
    artist: String,
    album: String,
    position: Option<u64>,
    duration: Option<u64>,
    pct: Option<u8>,
    is_playing: bool,
    last_update: u64, // 时间戳用于强制更新
}

type Shared = Arc<RwLock<Song>>;

#[tokio::main]
async fn main() {
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

    // 启动 Web 服务器
    let state: Shared = Arc::default();
    let st = state.clone();
    task::spawn_blocking(move || smtc_worker(st));

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
    
    // 在单独的线程中运行 Web 服务器
    let server_handle = std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            warp::serve(api.or(static_files))
                .run(([127, 0, 0, 1], port))
                .await;
        });
    });

    // 等待 Web 服务器线程结束（理论上不会到达这里）
    let _ = server_handle.join();
}

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
        }
        None => Err(warp::reject::not_found()),
    }
}

fn with_state(
    s: Shared,
) -> impl Filter<Extract = (Shared,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || s.clone())
}

// -------------------- 后台轮询 --------------------
fn smtc_worker(state: Shared) {
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
    use std::time::{SystemTime, UNIX_EPOCH};

    let manager = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .and_then(|f| f.get()) {
        Ok(m) => m,
        Err(_) => {
            eprintln!("Failed to get session manager");
            return;
        }
    };

    let mut last_song = Song::default();
    let mut last_position = None;
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
                current_song.is_playing = playback_info.PlaybackStatus().unwrap_or_default() == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
            }
            
            // 元数据
            if let Ok(info) = session.TryGetMediaPropertiesAsync()
                .and_then(|f| f.get()) {
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
                    current_song.position = Some(pos_s as u64);
                    current_song.duration = Some(dur_s as u64);
                    current_song.pct = Some(((pos_s * 100) / dur_s).min(100) as u8);
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
