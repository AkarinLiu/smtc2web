use crate::{log_debug, log_error, log_info, log_warn};
use base64::{engine::general_purpose::STANDARD, Engine};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub name: String,
    pub folder_name: String,
    pub author: String,
    pub version: String,
    pub screenshot_path: String,
    pub is_default: bool,
    pub is_builtin: bool,
}

#[derive(RustEmbed)]
#[folder = "frontend"]
struct DefaultTheme;

pub struct ThemeManager;

impl ThemeManager {
    /// 获取主题目录路径
    pub fn get_themes_dir() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("smtc2web");
        path.push("theme");
        path
    }

    /// 确保主题目录存在
    pub fn ensure_themes_dir() -> io::Result<()> {
        let themes_dir = Self::get_themes_dir();
        if !themes_dir.exists() {
            fs::create_dir_all(&themes_dir)?;
        }
        Ok(())
    }

    /// 获取默认主题信息
    fn get_default_theme_info() -> ThemeInfo {
        // 从嵌入的资源中读取 theme.toml
        let theme_toml_content = DefaultTheme::get("theme.toml")
            .map(|c| String::from_utf8_lossy(&c.data).to_string())
            .unwrap_or_else(|| {
                // 如果读取失败，使用硬编码的默认值
                r#"[smtc2web.theme]
name = "默认主题"
version = "0.1.1"
author = "AkarinLiu"
screenshot = "screenshot.png"
"#
                .to_string()
            });

        // 解析 TOML
        let config: toml::Value =
            toml::from_str(&theme_toml_content).expect("Failed to parse embedded theme.toml");

        // 尝试从 [smtc2web.theme] 节读取配置
        let theme_section = config.get("smtc2web").and_then(|s| s.get("theme"));
        let theme_config = theme_section.unwrap_or(&config);

        let name = theme_config
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("默认主题")
            .to_string();
        let author = theme_config
            .get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("AkarinLiu")
            .to_string();
        let version = theme_config
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.1")
            .to_string();
        let screenshot = theme_config
            .get("screenshot")
            .and_then(|v| v.as_str())
            .unwrap_or("screenshot.png")
            .to_string();

        // 从嵌入的资源中读取截图并转换为 Base64
        let screenshot_path = if let Some(screenshot_data) = DefaultTheme::get(&screenshot) {
            // 根据文件扩展名判断 MIME 类型
            let mime = if screenshot.ends_with(".png") {
                "image/png"
            } else if screenshot.ends_with(".jpg") || screenshot.ends_with(".jpeg") {
                "image/jpeg"
            } else {
                "image/png"
            };
            format!(
                "data:{};base64,{}",
                mime,
                STANDARD.encode(&screenshot_data.data)
            )
        } else {
            // 如果找不到截图，返回空字符串
            String::new()
        };

        ThemeInfo {
            name,
            folder_name: "default".to_string(),
            author,
            version,
            screenshot_path,
            is_default: true,
            is_builtin: true,
        }
    }

    /// 获取主题配置文件路径
    fn get_theme_config_path(theme_folder: &str) -> PathBuf {
        let mut path = Self::get_themes_dir();
        path.push(theme_folder);
        path.push("theme.toml");
        path
    }

    /// 解析主题配置
    fn parse_theme_config(theme_folder: &str) -> Option<ThemeInfo> {
        let config_path = Self::get_theme_config_path(theme_folder);

        if !config_path.exists() {
            log_debug!("  配置文件不存在: {:?}", config_path);
            return None;
        }

        let content = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(e) => {
                log_error!("  读取配置文件失败: {}", e);
                return None;
            }
        };

        let config: toml::Value = match toml::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                log_error!("  解析 TOML 失败: {}", e);
                return None;
            }
        };

        // 尝试从 [smtc2web.theme] 节读取配置
        let theme_section = config.get("smtc2web").and_then(|s| s.get("theme"));

        // 如果找不到嵌套节，尝试直接从顶层读取（向后兼容）
        let theme_config = theme_section.unwrap_or(&config);

        let name = theme_config.get("name")?.as_str()?.to_string();
        let author = theme_config.get("author")?.as_str()?.to_string();
        let version = theme_config.get("version")?.as_str()?.to_string();
        let screenshot = theme_config.get("screenshot")?.as_str()?.to_string();

        // 验证截图路径：禁止网络 URL，只允许本地相对路径
        let screenshot_path = if screenshot.starts_with("http://")
            || screenshot.starts_with("https://")
        {
            log_warn!("  主题 '{}' 的截图使用了网络 URL，已忽略", name);
            String::new()
        } else {
            // 构建截图的完整路径
            let mut full_path = Self::get_themes_dir();
            full_path.push(theme_folder);
            full_path.push(&screenshot);

            // 验证文件是否存在，如果存在则读取并转为 Base64
            if full_path.exists() && full_path.is_file() {
                match fs::read(&full_path) {
                    Ok(data) => {
                        // 根据文件扩展名判断 MIME 类型
                        let mime = if screenshot.ends_with(".png") {
                            "image/png"
                        } else if screenshot.ends_with(".jpg") || screenshot.ends_with(".jpeg") {
                            "image/jpeg"
                        } else {
                            "image/png"
                        };
                        format!("data:{};base64,{}", mime, STANDARD.encode(&data))
                    }
                    Err(e) => {
                        log_warn!("  读取主题 '{}' 的截图文件失败: {}", name, e);
                        String::new()
                    }
                }
            } else {
                log_warn!("  主题 '{}' 的截图文件不存在: {:?}", name, full_path);
                String::new()
            }
        };

        Some(ThemeInfo {
            name,
            folder_name: theme_folder.to_string(),
            author,
            version,
            screenshot_path,
            is_default: false,
            is_builtin: false,
        })
    }

    /// 扫描所有主题
    pub fn scan_themes() -> io::Result<Vec<ThemeInfo>> {
        Self::ensure_themes_dir()?;

        let themes_dir = Self::get_themes_dir();
        let mut themes = Vec::new();

        // 首先添加默认主题
        let default_theme = Self::get_default_theme_info();
        themes.push(default_theme);

        log_info!("扫描主题目录: {:?}", themes_dir);

        if let Ok(entries) = fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(folder_name) = path.file_name() {
                        let folder_name = folder_name.to_string_lossy().to_string();

                        // 跳过 default 文件夹（默认主题使用内置的）
                        if folder_name == "default" {
                            log_debug!("发现文件夹: {} (跳过，使用内置默认主题)", folder_name);
                            continue;
                        }

                        log_debug!("发现文件夹: {}", folder_name);

                        let config_path = Self::get_theme_config_path(&folder_name);
                        log_debug!("  配置文件路径: {:?}", config_path);
                        log_debug!("  配置文件存在: {}", config_path.exists());

                        if let Some(theme_info) = Self::parse_theme_config(&folder_name) {
                            log_info!("  成功解析主题: {}", theme_info.name);
                            themes.push(theme_info);
                        } else {
                            log_debug!("  解析主题失败，跳过");
                        }
                    }
                }
            }
        }

        log_info!("扫描完成，共找到 {} 个主题", themes.len());
        Ok(themes)
    }

    /// 解压主题压缩包
    pub fn extract_theme(zip_path: &Path) -> io::Result<String> {
        Self::ensure_themes_dir()?;

        let file = fs::File::open(zip_path)?;
        let mut archive = ZipArchive::new(file)?;

        // 获取主题名称（使用压缩包文件名，不含扩展名）
        let theme_name = zip_path
            .file_stem()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "无效的文件名"))?
            .to_string_lossy()
            .to_string();

        // 禁止覆盖默认主题
        if theme_name == "default" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "不能覆盖默认主题",
            ));
        }

        let themes_dir = Self::get_themes_dir();
        let theme_dir = themes_dir.join(&theme_name);

        // 如果目录已存在，先删除
        if theme_dir.exists() {
            fs::remove_dir_all(&theme_dir)?;
        }
        fs::create_dir_all(&theme_dir)?;

        // 检测 ZIP 根目录结构
        // 如果所有文件都在同一个根文件夹内，则去掉这层文件夹
        let mut root_folder: Option<String> = None;
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let file_path = file.name();

            // 跳过不安全的文件名
            if file_path.contains("..") || file_path.starts_with('/') {
                continue;
            }

            // 获取第一级目录名
            let parts: Vec<&str> = file_path.splitn(2, '/').collect();
            if parts.len() > 1 {
                let first_part = parts[0].to_string();
                match &root_folder {
                    None => root_folder = Some(first_part),
                    Some(root) if root == &first_part => {}
                    _ => {
                        // 有多个不同的根文件夹，不使用根文件夹剥离
                        root_folder = None;
                        break;
                    }
                }
            } else {
                // 有文件直接在根目录，不使用根文件夹剥离
                root_folder = None;
                break;
            }
        }

        // 重新打开 archive（因为上面已经遍历过）
        let file = fs::File::open(zip_path)?;
        let mut archive = ZipArchive::new(file)?;

        // 解压文件
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_path = file.name();

            // 跳过不安全的文件名
            if file_path.contains("..") || file_path.starts_with('/') {
                continue;
            }

            // 去掉根文件夹（如果存在且统一）
            let relative_path = match &root_folder {
                Some(root) => {
                    if file_path.starts_with(root) {
                        file_path
                            .strip_prefix(root)
                            .unwrap_or(file_path)
                            .strip_prefix('/')
                            .unwrap_or(file_path.strip_prefix(root).unwrap_or(file_path))
                    } else {
                        file_path
                    }
                }
                None => file_path,
            };

            // 跳过空路径
            if relative_path.is_empty() {
                continue;
            }

            let out_path = theme_dir.join(relative_path);

            if file.is_dir() {
                fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = fs::File::create(&out_path)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(theme_name)
    }

    /// 删除主题
    pub fn delete_theme(theme_folder: &str) -> io::Result<()> {
        // 禁止删除默认主题
        if theme_folder == "default" {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "不能删除默认主题",
            ));
        }

        let themes_dir = Self::get_themes_dir();
        let theme_dir = themes_dir.join(theme_folder);

        if theme_dir.exists() {
            fs::remove_dir_all(&theme_dir)?;
        }

        Ok(())
    }

    /// 获取主题的服务路径
    pub fn get_theme_server_path(theme_folder: &str) -> PathBuf {
        // 默认主题使用空路径（表示使用内置主题）
        if theme_folder == "default" {
            return PathBuf::new();
        }
        Self::get_themes_dir().join(theme_folder)
    }
}
