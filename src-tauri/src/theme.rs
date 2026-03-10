use rust_embed::RustEmbed;
use std::path::PathBuf;
use warp::{Filter, path::Tail};

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

    pub async fn serve_theme_file(self, tail: Tail) -> Result<impl warp::Reply, warp::Rejection> {
        let path = tail.as_str();
        let path = if path.is_empty() { "index.html" } else { path };

        // 首先尝试从自定义主题路径加载文件
        // 只有当主题路径有效且不为空时才尝试读取自定义主题
        let has_custom_theme = !self.theme_path.to_string_lossy().is_empty()
            && self.theme_path.components().next().is_some();

        if has_custom_theme {
            let custom_path = self.theme_path.join(path);
            // 通过规范化路径并确保其仍然位于主题目录下，防止目录遍历
            if let (Ok(base_dir), Ok(resolved_path)) = (
                std::fs::canonicalize(&self.theme_path),
                std::fs::canonicalize(&custom_path),
            ) && resolved_path.starts_with(&base_dir)
                && let Ok(content) = std::fs::read(&resolved_path)
            {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                return Ok(warp::reply::with_header(
                    content,
                    "content-type",
                    mime.as_ref(),
                ));
            }
        }

        // 否则使用默认嵌入的主题文件
        match DefaultTheme::get(path) {
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

    pub fn with_manager(
        manager: Self,
    ) -> impl Filter<Extract = (Self,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || manager.clone())
    }
}
