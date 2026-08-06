use crate::cli::DevArgs;
use crate::config::Config;
use crate::logger;
#[cfg(target_os = "linux")]
use crate::media::{MediaSession, PlatformSession};
use crate::{Shared, Song, log_error, log_info, log_warn};
use axum::Router;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::routing::{any, get};
use notify::{
    Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};

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

/* ---------- 开发模式媒体 (Windows: 事件驱动, Linux: 轮询) ---------- */
fn dev_media_worker(state: Shared, process_filter: String) {
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = crate::media::smtc::run_event_driven(state, &process_filter) {
            eprintln!("错误: Dev media event worker failed: {}", e);
            log_error!("Dev media event worker failed: {}", e);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let session = match PlatformSession::new(&process_filter) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("错误: 创建媒体会话失败: {}", e);
                log_error!("创建媒体会话失败: {}", e);
                return;
            }
        };

        crate::media::poll_media_loop(
            &session,
            &state,
            &crate::CURRENT_APP_ID,
            &crate::CURRENT_APP_DISPLAY_NAME,
        );
    }
}

/* ---------- 静态模式：文件服务 ---------- */
async fn serve_dev_file(
    State(theme_dir): State<PathBuf>,
    req: Request,
) -> Result<axum::response::Response, StatusCode> {
    let path = req.uri().path().trim_start_matches('/').to_string();
    let path = if path.is_empty() { "index.html" } else { &path };
    let file_path = theme_dir.join(path);

    let canonical_base = std::fs::canonicalize(&theme_dir).map_err(|_| StatusCode::NOT_FOUND)?;
    let resolved_path = std::fs::canonicalize(&file_path).map_err(|_| StatusCode::NOT_FOUND)?;

    if !resolved_path.starts_with(&canonical_base) {
        return Err(StatusCode::NOT_FOUND);
    }

    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    let data = std::fs::read(&resolved_path).map_err(|_| StatusCode::NOT_FOUND)?;

    let body = if mime.starts_with("text/html") {
        let html = String::from_utf8_lossy(&data);
        inject_sse(&html).into_bytes()
    } else {
        data
    };

    Ok(crate::theme::response_with_content_type(body, &mime))
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
struct ProxyCtx {
    client: reqwest::Client,
    vite_port: u16,
}

async fn proxy_to_vite(
    State(ctx): State<Arc<ProxyCtx>>,
    req: Request,
) -> Result<axum::response::Response, StatusCode> {
    let path = req.uri().path().trim_start_matches('/').to_string();
    let url = format!("http://127.0.0.1:{}/{}", ctx.vite_port, path);
    let resp = ctx
        .client
        .get(&url)
        .send()
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let body = resp
        .bytes()
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?
        .to_vec();

    Ok(crate::theme::response_with_content_type(body, &ct))
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
                Ok(Ok(event))
                    if matches!(
                        event.kind,
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                    ) =>
                {
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
            println!("Vite dev server 启动中 (npx vite) ...");
            log_info!("Vite dev server 启动中 (npx vite) ...");
            wait_for_vite_ready(vite_port).await;
            Some(child)
        }
        Err(e) => {
            eprintln!("警告: npx 启动失败: {}", e);
            log_warn!("npx 启动失败: {}", e);
            let bin =
                theme_dir
                    .join("node_modules")
                    .join(".bin")
                    .join(if cfg!(target_os = "windows") {
                        "vite.cmd"
                    } else {
                        "vite"
                    });
            if !bin.exists() {
                eprintln!("错误: 未找到 Vite 可执行文件");
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
                    println!("Vite dev server 启动中 ...");
                    log_info!("Vite dev server 启动中 ...");
                    wait_for_vite_ready(vite_port).await;
                    Some(child)
                }
                Err(e) => {
                    eprintln!("错误: 启动 Vite 失败: {}", e);
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
            println!("Vite dev server 就绪: {}", url);
            log_info!("Vite dev server 就绪: {}", url);
            return;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    eprintln!("警告: Vite dev server 启动超时, 端口 {}", vite_port);
    log_warn!("Vite dev server 启动超时, 端口 {}", vite_port);
}

async fn dev_api_now(State(state): State<Shared>) -> Json<Song> {
    Json(state.read().unwrap().clone())
}

/* ---------- SSE 重载路由 ---------- */
async fn sse_reload_route(
    State(reload_tx): State<broadcast::Sender<()>>,
) -> Sse<impl Stream<Item = Result<SseEvent, std::convert::Infallible>>> {
    let stream = BroadcastStream::new(reload_tx.subscribe()).map(|r| match r {
        Ok(()) => Ok(SseEvent::default().data("reload")),
        Err(_) => Ok(SseEvent::default().data("reload")),
    });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

/* ---------- Vite 子进程退出监控 ---------- */
async fn monitor_vite_child(vite_child: Arc<tokio::sync::Mutex<Option<tokio::process::Child>>>) {
    let child_opt = vite_child.lock().await.take();
    if let Some(mut child) = child_opt {
        let status = child.wait().await;
        let exit_code = status.as_ref().ok().and_then(|s| s.code());
        eprintln!("警告: Vite dev server 已退出 (exit: {:?})", exit_code);
        log_warn!("Vite dev server 已退出 (exit: {:?})", exit_code);
    } else {
        std::future::pending::<()>().await;
    }
}

/* ==================== 主入口 ==================== */

pub async fn run(args: DevArgs) {
    logger::init();
    println!("smtc2web dev - 主题开发服务器");
    log_info!("smtc2web dev - 主题开发服务器");

    // 1. 验证主题目录
    let theme_dir = match std::fs::canonicalize(&args.path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: 无效的主题目录 '{}': {}", args.path.display(), e);
            log_error!("无效的主题目录 '{}': {}", args.path.display(), e);
            std::process::exit(1);
        }
    };
    if !theme_dir.join("theme.toml").exists() {
        eprintln!("错误: 未找到 theme.toml: {}", theme_dir.display());
        log_error!("未找到 theme.toml: {}", theme_dir.display());
        std::process::exit(1);
    }

    // 2. 打印主题信息
    if let Some((name, version, author)) = parse_theme_info(&theme_dir) {
        println!("{} {} v{} by {}", "=".repeat(40), name, version, author);
    }

    // 3. Vite 检测
    let use_vite = args.vite || VITE_CONFIG_FILES.iter().any(|f| theme_dir.join(f).exists());

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
        println!("检测到 Vite 项目, 启动 Vite dev server...");
        log_info!("检测到 Vite 项目, 启动 Vite dev server...");
        let child = start_vite_process(&theme_dir, args.vite_port).await;
        let has = child.is_some();
        *vite_child.lock().await = child;
        if has {
            println!();
            println!("  Vite 模式已启用");
            println!("  请在 vite.config 中添加代理:");
            println!(
                "    server: {{ proxy: {{ '/api': 'http://localhost:{}' }} }}",
                args.port
            );
            println!();
        }
    }

    let vite_active = { use_vite && vite_child.lock().await.is_some() };

    // 8. 路由 + 服务器
    let api = Router::new()
        .route("/api/now", get(dev_api_now))
        .with_state(state.clone());
    let sse = Router::new()
        .route("/__dev_reload", get(sse_reload_route))
        .with_state(reload_tx.clone());

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let server_handle: tokio::task::JoinHandle<()> = if use_vite && vite_active {
        let ctx = Arc::new(ProxyCtx {
            client: reqwest::Client::new(),
            vite_port: args.vite_port,
        });
        let proxy = Router::new().fallback(any(proxy_to_vite)).with_state(ctx);
        tokio::spawn(async move {
            let listener = TcpListener::bind((address, args.port))
                .await
                .expect("failed to bind dev server");
            let _ = axum::serve(listener, api.merge(sse).merge(proxy))
                .with_graceful_shutdown(async {
                    let _ = rx.await;
                })
                .await;
        })
    } else {
        let serve = Router::new()
            .fallback(any(serve_dev_file))
            .with_state(theme_dir.clone());
        tokio::spawn(async move {
            let listener = TcpListener::bind((address, args.port))
                .await
                .expect("failed to bind dev server");
            let _ = axum::serve(listener, api.merge(sse).merge(serve))
                .with_graceful_shutdown(async {
                    let _ = rx.await;
                })
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
    println!("  Dev server: http://{}:{}", address, args.port);
    println!("  Theme:      {}", theme_dir.display());
    log_info!("  Dev server: http://{}:{}", address, args.port);
    log_info!("  Theme:      {}", theme_dir.display());
    if use_vite && vite_active {
        println!("  Vite:       http://127.0.0.1:{}", args.vite_port);
        log_info!("  Vite:       http://127.0.0.1:{}", args.vite_port);
    } else if !use_vite {
        println!("  文件监控已启用");
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
            eprintln!("警告: 打开浏览器失败: {}", e);
            log_warn!("打开浏览器失败: {}", e);
        }
    }

    // 11. 等待退出
    let monitor = monitor_vite_child(vite_child.clone());
    tokio::pin!(monitor);

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!();
            println!("收到退出信号, 正在关闭...");
            log_info!("收到退出信号, 正在关闭...");
        }
        _ = &mut monitor => {
            println!();
            println!("Vite 进程已退出, 正在关闭...");
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
    println!("开发服务器已关闭");
    log_info!("开发服务器已关闭");
}
