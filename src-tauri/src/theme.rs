use rust_embed::RustEmbed;
use std::path::PathBuf;
use warp::{Filter, path::Tail};

#[derive(RustEmbed)]
#[folder = "frontend"]
struct DefaultTheme;

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
        if !self.theme_path.as_os_str().is_empty() {
            let custom_path = self.theme_path.join(path);
            if let Ok(content) = std::fs::read(&custom_path) {
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
