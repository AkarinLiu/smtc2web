use crate::cli::DevArgs;
use crate::config::Config;
use crate::logger;
use crate::media::{
    generate_song_id, get_cached_album_art, set_cached_album_art, MediaSession, PlatformSession,
};
use crate::{format_duration, log_error, log_info, log_warn, Shared, Song};
use notify::{Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use warp::Filter;

/* ---------- SSE 热重载脚本 ---------- */
const SSE_RELOAD_SCRIPT: &str = r#"<script>
(function(){var e=new EventSource('/__dev_reload');e.addEventListener('reload',function(){e.close();location.reload()});e.onerror=function(){e.close()}})();
</script>"#;

const BODY_CLOSE_TAG: &str = "</body>";

/* ---------- Vite 配置文件检测 ---------- */
const VITE_CONFIG_FILES: &[&str] = &["vite.config.ts", "vite.config.js", "vite.config.mjs"];

/* ---------- 主题信息解析 ---------- */
fn parse_theme_info(theme_dir: &Path) -> Option<(String, String, String)> {
    let theme_toml = theme_dir.join("theme.toml");
    let content = std::fs::read_to_string(&theme_toml).ok()?;
    let toml_value: toml::Value = toml::from_str(&content).ok()?;
    let theme_section = toml_value
        .get("smtc2web")
        .and_then(|s| s.get("theme"))
        .unwrap_or(&toml_value);

    Some((
        theme_section
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string(),
        theme_section
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string(),
        theme_section
            .get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string(),
    ))
}

/* ---------- 开发模式媒体轮询 ---------- */
fn dev_media_worker(state: Shared, process_filter: String) {
    let session = match PlatformSession::new(&process_filter) {
        Ok(s) => s,
        Err(e) => {
            log_error!("创建媒体会话失败: {}", e);
            return;
        }
    };

    let mut last_song = Song::default();
    let mut last_position: Option<String> = None;
    let mut last_song_id = String::new();
    let mut last_art_update: u64 = 0;

    loop {
        let mut current_song = Song::default();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        current_song.last_update = timestamp;

        if let Some(info) = session.poll_current() {
            current_song.is_playing = info.is_playing;
            current_song.title = info.title;
            current_song.artist = info.artist;
            current_song.album = info.album;

            let current_song_id =
                generate_song_id(&current_song.title, &current_song.artist, &current_song.album);
            let cached_art = get_cached_album_art(&current_song_id);

            let should_fetch_art = cached_art.is_none()
                || (current_song_id != last_song_id
                    && timestamp.saturating_sub(last_art_update) > 30);

            if should_fetch_art {
                current_song.album_art = session.get_album_art_base64(
                    &current_song.artist,
                    &current_song.title,
                    &current_song.album,
                );
                if let Some(ref art) = current_song.album_art {
                    set_cached_album_art(&current_song_id, art.clone());
                }
                last_song_id = current_song_id;
                last_art_update = timestamp;
            } else {
                current_song.album_art = cached_art;
            }

            if info.duration_secs > 0 {
                current_song.position = Some(format_duration(info.position_secs));
                current_song.duration = Some(format_duration(info.duration_secs));
                let pct = (info.position_secs as f64 * 100.0) / info.duration_secs as f64;
                current_song.pct = Some((pct * 10.0).round() / 10.0);
            }
        } else {
            let empty = Song::default();
            let mut s = state.write().unwrap();
            *s = empty.clone();
            last_song = empty;
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

        std::thread::sleep(if current_song.is_playing {
            Duration::from_millis(200)
        } else {
            Duration::from_millis(1000)
        });
    }
}

/* ---------- 静态模式：文件服务 ---------- */
async fn serve_dev_file(
    path: &str,
    theme_dir: &Path,
) -> Result<warp::http::Response<Vec<u8>>, warp::Rejection> {
    let path = if path.is_empty() { "index.html" } else { path };
    let file_path = theme_dir.join(path);

    let canonical_base =
        std::fs::canonicalize(theme_dir).map_err(|_| warp::reject::not_found())?;
    let resolved_path =
        std::fs::canonicalize(&file_path).map_err(|_| warp::reject::not_found())?;

    if !resolved_path.starts_with(&canonical_base) {
        return Err(warp::reject::not_found());
    }

    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    let data = std::fs::read(&resolved_path).map_err(|_| warp::reject::not_found())?;

    let body = if mime.starts_with("text/html") {
        let html = String::from_utf8_lossy(&data);
        inject_sse(&html).into_bytes()
    } else {
        data
    };

    let mut response = warp::http::Response::new(body);
    response.headers_mut().insert(
        "content-type",
        warp::http::HeaderValue::from_str(&mime).unwrap(),
    );
    Ok(response)
}

fn inject_sse(html: &str) -> String {
    if let Some(pos) = html.to_lowercase().rfind(BODY_CLOSE_TAG) {
        let mut s = html.to_string();
        s.insert_str(pos, SSE_RELOAD_SCRIPT);
        s
    } else {
        format!("{}{}", html, SSE_RELOAD_SCRIPT)
    }
}

/* ---------- Vite 反向代理 ---------- */
async fn proxy_to_vite(
    path: &str,
    vite_port: u16,
    client: &reqwest::Client,
) -> Result<warp::http::Response<Vec<u8>>, warp::Rejection> {
    let url = format!("http://127.0.0.1:{}/{}", vite_port, path);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|_| warp::reject::not_found())?;

    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let body = resp
        .bytes()
        .await
        .map_err(|_| warp::reject::not_found())?
        .to_vec();

    let mut response = warp::http::Response::new(body);
    response
        .headers_mut()
        .insert("content-type", warp::http::HeaderValue::from_str(&ct).unwrap());
    Ok(response)
}

/* ---------- 文件监控 ---------- */
fn start_file_watcher(
    theme_dir: &Path,
    reload_tx: broadcast::Sender<()>,
) -> Option<RecommendedWatcher> {
    let theme_dir = theme_dir.to_path_buf();
    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<Event>>();

    let mut watcher = match RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        NotifyConfig::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            log_warn!("文件监控初始化失败: {}", e);
            return None;
        }
    };

    if let Err(e) = watcher.watch(&theme_dir, RecursiveMode::Recursive) {
        log_warn!("文件监控启动失败: {}", e);
        return None;
    }

    std::thread::spawn(move || {
        let mut pending = false;
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) if matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                ) => {
                    pending = true;
                    while let Ok(Ok(_)) = rx.try_recv() {}
                    if pending {
                        let _ = reload_tx.send(());
                        pending = false;
                    }
                }
                Ok(Err(_)) | Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if pending {
                        let _ = reload_tx.send(());
                        pending = false;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                _ => {}
            }
        }
    });

    Some(watcher)
}

/* ---------- Vite 子进程 ---------- */
async fn start_vite_process(theme_dir: &Path, vite_port: u16) -> Option<tokio::process::Child> {
    let result = tokio::process::Command::new("npx")
        .args(["vite", "--port", &vite_port.to_string(), "--strictPort"])
        .current_dir(theme_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn();

    match result {
        Ok(child) => {
            log_info!("Vite dev server 启动中 (npx vite) ...");
            wait_for_vite_ready(vite_port).await;
            Some(child)
        }
        Err(e) => {
            log_warn!("npx 启动失败: {}", e);
            let bin = theme_dir.join("node_modules").join(".bin").join(
                if cfg!(target_os = "windows") {
                    "vite.cmd"
                } else {
                    "vite"
                },
            );
            if !bin.exists() {
                log_error!("未找到 Vite 可执行文件");
                return None;
            }
            match tokio::process::Command::new(&bin)
                .args(["--port", &vite_port.to_string(), "--strictPort"])
                .current_dir(theme_dir)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(child) => {
                    log_info!("Vite dev server 启动中 ...");
                    wait_for_vite_ready(vite_port).await;
                    Some(child)
                }
                Err(e) => {
                    log_error!("启动 Vite 失败: {}", e);
                    None
                }
            }
        }
    }
}

async fn wait_for_vite_ready(vite_port: u16) {
    let url = format!("http://127.0.0.1:{}/", vite_port);
    let client = reqwest::Client::new();
    for _ in 0..30 {
        if client.head(&url).send().await.is_ok() {
            log_info!("Vite dev server 就绪: {}", url);
            return;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    log_warn!("Vite dev server 启动超时, 端口 {}", vite_port);
}

/* ---------- SSE 重载路由 ---------- */
fn sse_reload_route(
    reload_tx: broadcast::Sender<()>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("__dev_reload").and(warp::get()).map(move || {
        let rx = reload_tx.subscribe();
        let stream =
            tokio_stream::wrappers::BroadcastStream::new(rx).map(|r| match r {
                Ok(()) => Ok::<_, warp::Error>(warp::sse::Event::default().data("reload")),
                Err(_) => Ok(warp::sse::Event::default().data("reload")),
            });
        warp::sse::reply(warp::sse::keep_alive().stream(stream))
    })
}

/* ---------- Vite 子进程退出监控 ---------- */
async fn monitor_vite_child(vite_child: Arc<tokio::sync::Mutex<Option<tokio::process::Child>>>) {
    let child_opt = vite_child.lock().await.take();
    if let Some(mut child) = child_opt {
        let status = child.wait().await;
        log_warn!(
            "Vite dev server 已退出 (exit: {:?})",
            status.map(|s| s.code())
        );
    } else {
        std::future::pending::<()>().await;
    }
}

/* ==================== 主入口 ==================== */

pub async fn run(args: DevArgs) {
    logger::init();
    log_info!("smtc2web dev - 主题开发服务器");

    // 1. 验证主题目录
    let theme_dir = match std::fs::canonicalize(&args.path) {
        Ok(d) => d,
        Err(e) => {
            log_error!("无效的主题目录 '{}': {}", args.path.display(), e);
            std::process::exit(1);
        }
    };
    if !theme_dir.join("theme.toml").exists() {
        log_error!("未找到 theme.toml: {}", theme_dir.display());
        std::process::exit(1);
    }

    // 2. 打印主题信息
    if let Some((name, version, author)) = parse_theme_info(&theme_dir) {
        println!("{} {} v{} by {}", "=".repeat(40), name, version, author);
    }

    // 3. Vite 检测
    let use_vite = args.vite
        || VITE_CONFIG_FILES
            .iter()
            .any(|f| theme_dir.join(f).exists());

    // 4. 媒体轮询
    let state: Shared = Arc::default();
    let pf = Config::load()
        .map(|c| c.process_filter)
        .unwrap_or_else(|_| "*".to_string());
    std::thread::spawn({
        let s = state.clone();
        move || dev_media_worker(s, pf)
    });

    // 5. 热重载通道
    let (reload_tx, _) = broadcast::channel::<()>(16);

    // 6. 地址
    let address = Config::load()
        .ok()
        .and_then(|c| c.address.parse().ok())
        .unwrap_or(IpAddr::from([127, 0, 0, 1]));

    // 7. Vite 子进程
    let vite_child: Arc<tokio::sync::Mutex<Option<tokio::process::Child>>> =
        Arc::new(tokio::sync::Mutex::new(None));

    if use_vite {
        println!();
        log_info!("检测到 Vite 项目, 启动 Vite dev server...");
        let child = start_vite_process(&theme_dir, args.vite_port).await;
        let has = child.is_some();
        *vite_child.lock().await = child;
        if has {
            println!();
            println!("  Vite 模式已启用");
            println!("  请在 vite.config 中添加代理:");
            println!("    server: {{ proxy: {{ '/api': 'http://localhost:{}' }} }}", args.port);
            println!();
        }
    }

    let vite_active = { use_vite && vite_child.lock().await.is_some() };

    // 8. 路由 + 服务器
    let state_filter = warp::any().map({
        let s = state.clone();
        move || s.clone()
    });
    let api =
        warp::path!("api" / "now")
            .and(state_filter)
            .map(|s: Shared| warp::reply::json(&*s.read().unwrap()));

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let server_handle: tokio::task::JoinHandle<()> = if use_vite && vite_active {
        let vp = args.vite_port;
        let client = reqwest::Client::new();
        let proxy = warp::path::tail().and_then(move |t: warp::path::Tail| {
            let c = client.clone();
            let p = t.as_str().to_string();
            async move { proxy_to_vite(&p, vp, &c).await }
        });
        let routes = api.or(proxy).boxed();
        tokio::spawn(async move {
            warp::serve(routes)
                .bind_with_graceful_shutdown((address, args.port), async {
                    let _ = rx.await;
                })
                .1
                .await;
        })
    } else {
        let td = theme_dir.clone();
        let serve = warp::path::tail().and_then(move |t: warp::path::Tail| {
            let d = td.clone();
            let p = t.as_str().to_string();
            async move { serve_dev_file(&p, &d).await }
        });
        let sse = sse_reload_route(reload_tx.clone());
        let routes = api.or(sse).or(serve).boxed();
        tokio::spawn(async move {
            warp::serve(routes)
                .bind_with_graceful_shutdown((address, args.port), async {
                    let _ = rx.await;
                })
                .1
                .await;
        })
    };

    // 9. 文件监控（静态模式）
    let _watcher = if !use_vite {
        start_file_watcher(&theme_dir, reload_tx)
    } else {
        None
    };

    // 10. 信息输出 & 打开浏览器
    println!();
    log_info!("  Dev server: http://{}:{}", address, args.port);
    log_info!("  Theme:      {}", theme_dir.display());
    if use_vite && vite_active {
        log_info!("  Vite:       http://127.0.0.1:{}", args.vite_port);
    } else if !use_vite {
        log_info!("  文件监控已启用");
    }
    println!();

    if !args.no_open {
        let url = if use_vite && vite_active {
            format!("http://127.0.0.1:{}", args.vite_port)
        } else {
            format!("http://{}:{}", address, args.port)
        };
        if let Err(e) = open::that(&url) {
            log_warn!("打开浏览器失败: {}", e);
        }
    }

    // 11. 等待退出
    let monitor = monitor_vite_child(vite_child.clone());
    tokio::pin!(monitor);

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!();
            log_info!("收到退出信号, 正在关闭...");
        }
        _ = &mut monitor => {
            println!();
            log_info!("Vite 进程已退出, 正在关闭...");
        }
    }

    // 12. 清理
    {
        let mut g = vite_child.lock().await;
        if let Some(ref mut c) = *g {
            let _ = c.kill().await;
        }
    }
    let _ = tx.send(());
    let _ = tokio::time::timeout(Duration::from_secs(3), server_handle).await;
    log_info!("开发服务器已关闭");
}
