use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// 托盘菜单翻译结构
#[derive(Debug, Deserialize, Clone)]
pub struct TrayTranslations {
    pub show_window: String,
    pub open_web: String,
    pub quit: String,
}

/// 语言包结构
#[derive(Debug, Deserialize, Clone)]
pub struct Locale {
    pub tray: TrayTranslations,
}

/// 当前语言状态
pub static CURRENT_LOCALE: once_cell::sync::Lazy<Mutex<String>> =
    once_cell::sync::Lazy::new(|| Mutex::new("zh-CN".to_string()));

/// 缓存的语言包
pub static LOCALE_CACHE: once_cell::sync::Lazy<Mutex<HashMap<String, Locale>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

/// 加载指定语言的翻译文件
pub fn load_locale(locale: &str) -> Option<Locale> {
    // 首先检查缓存
    {
        let cache = LOCALE_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(locale) {
            return Some(cached.clone());
        }
    }

    // 获取语言文件路径
    let path = get_locale_path(locale)?;

    // 读取并解析 TOML 文件
    let content = fs::read_to_string(&path).ok()?;
    let locale_data: Locale = toml::from_str(&content).ok()?;

    // 存入缓存
    {
        let mut cache = LOCALE_CACHE.lock().unwrap();
        cache.insert(locale.to_string(), locale_data.clone());
    }

    Some(locale_data)
}

/// 获取语言文件路径
fn get_locale_path(locale: &str) -> Option<PathBuf> {
    // 尝试从应用资源目录加载
    if let Ok(app_dir) = std::env::current_exe() {
        let locales_dir = app_dir
            .parent()?
            .join("locales")
            .join(format!("{}.toml", locale));
        if locales_dir.exists() {
            return Some(locales_dir);
        }
    }

    // 开发模式下从项目目录加载
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    let path = PathBuf::from(manifest_dir)
        .join("locales")
        .join(format!("{}.toml", locale));
    if path.exists() {
        return Some(path);
    }

    None
}

/// 设置当前语言
pub fn set_locale(locale: &str) -> Result<(), String> {
    // 验证语言是否支持
    if !is_locale_supported(locale) {
        return Err(format!("Unsupported locale: {}", locale));
    }

    // 加载语言包（验证能否加载）
    load_locale(locale).ok_or_else(|| format!("Failed to load locale: {}", locale))?;

    // 更新当前语言
    let mut current = CURRENT_LOCALE.lock().unwrap();
    *current = locale.to_string();

    Ok(())
}

/// 获取当前语言
pub fn get_current_locale() -> String {
    CURRENT_LOCALE.lock().unwrap().clone()
}

/// 获取当前语言包
pub fn get_current_locale_data() -> Option<Locale> {
    let locale = get_current_locale();
    load_locale(&locale)
}

/// 检查语言是否支持
pub fn is_locale_supported(locale: &str) -> bool {
    matches!(locale, "zh-CN" | "en")
}

/// 获取默认语言
#[allow(dead_code)]
pub fn get_default_locale() -> &'static str {
    "zh-CN"
}
