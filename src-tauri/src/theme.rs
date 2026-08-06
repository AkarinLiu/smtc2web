use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "frontend"]
pub struct DefaultTheme;

#[derive(Clone)]
pub struct ThemeManager {
    theme_path: PathBuf,
}

impl ThemeManager {
    pub fn new(theme_path: &str) -> Self {
        Self {
            theme_path: PathBuf::from(theme_path),
        }
    }
}

pub(crate) fn response_with_content_type(body: Vec<u8>, mime: &str) -> Response {
    let mut response = body.into_response();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_str(mime).unwrap());
    response
}

pub async fn serve_theme_file(
    State(manager): State<ThemeManager>,
    req: Request,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().trim_start_matches('/').to_string();
    let path = if path.is_empty() { "index.html" } else { &path };

    // 首先尝试从自定义主题路径加载文件
    // 只有当主题路径有效且不为空时才尝试读取自定义主题
    let has_custom_theme = !manager.theme_path.to_string_lossy().is_empty()
        && manager.theme_path.components().next().is_some();

    if has_custom_theme {
        let custom_path = manager.theme_path.join(path);
        // 通过规范化路径并确保其仍然位于主题目录下，防止目录遍历
        if let (Ok(base_dir), Ok(resolved_path)) = (
            std::fs::canonicalize(&manager.theme_path),
            std::fs::canonicalize(&custom_path),
        ) && resolved_path.starts_with(&base_dir)
            && let Ok(content) = std::fs::read(&resolved_path)
        {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            return Ok(response_with_content_type(content, mime.essence_str()));
        }
    }

    // 否则使用默认嵌入的主题文件
    match DefaultTheme::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Ok(response_with_content_type(
                content.data.to_vec(),
                mime.essence_str(),
            ))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}
