use serde::Serialize;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::task;
use warp::Filter;

#[derive(Default, Clone, Serialize)]
struct Song {
    title: String,
    artist: String,
    album: String,
    position: Option<u64>,
    duration: Option<u64>,
    pct: Option<u8>,
}

type Shared = Arc<RwLock<Song>>;

#[tokio::main]
async fn main() {
    let state: Shared = Arc::default();

    // 后台轮询
    let st = state.clone();
    task::spawn_blocking(move || smtc_worker(st));

    // JSON 接口
    let api = warp::path!("api" / "now")
        .and(with_state(state.clone()))
        .map(|s: Shared| warp::reply::json(&*s.read().unwrap()));

    // 静态文件托管（Vue 3 前端）
    let fe = warp::fs::dir("frontend");

    println!("Server running at http://localhost:3030");
    warp::serve(api.or(fe)).run(([127, 0, 0, 1], 3030)).await;
}

fn with_state(
    s: Shared,
) -> impl Filter<Extract = (Shared,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || s.clone())
}

// -------------------- 后台轮询 --------------------
fn smtc_worker(state: Shared) {
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .unwrap()
        .get()
        .unwrap();

    loop {
        if let Ok(session) = manager.GetCurrentSession() {
            // 1. 元数据
            if let Ok(info) = session.TryGetMediaPropertiesAsync().unwrap().get() {
                let mut s = state.write().unwrap();
                s.title = info.Title().unwrap_or_default().to_string();
                s.artist = info.Artist().unwrap_or_default().to_string();
                s.album = info.AlbumTitle().unwrap_or_default().to_string();
            }

            // 2. 进度（可选）
            if let Ok(timeline) = session.GetTimelineProperties() {
                let pos = timeline.Position().unwrap().Duration;
                let dur = timeline.EndTime().unwrap().Duration;
                let pos_s = pos / 10_000_000;
                let dur_s = dur / 10_000_000;

                let mut s = state.write().unwrap();
                if dur != 0 {
                    s.position = Some(pos_s as u64);
                    s.duration = Some(dur_s as u64);
                    s.pct = Some(((pos_s * 100) / dur_s).min(100) as u8);
                } else {
                    s.position = None;
                    s.duration = None;
                    s.pct = None;
                }
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}
