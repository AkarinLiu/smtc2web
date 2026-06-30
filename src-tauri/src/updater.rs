use crate::{log_error, log_info, APP_STATE};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

// ---------- 更新清单 JSON 结构 ----------

/// Tauri updater 兼容的 JSON 更新清单
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct UpdateManifest {
    pub version: String,
    pub notes: Option<String>,
    pub pub_date: Option<String>,
    pub platforms: std::collections::HashMap<String, PlatformUpdate>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PlatformUpdate {
    pub signature: String,
    pub url: String,
}

/// 返回给前端的检查结果
#[derive(Debug, Serialize, Clone)]
pub struct UpdateCheckResult {
    /// 是否有可用更新
    pub has_update: bool,
    /// 当前版本
    pub current_version: String,
    /// 最新版本
    pub latest_version: String,
    /// 更新说明
    pub notes: Option<String>,
    /// 下载地址
    pub download_url: Option<String>,
    /// 使用的更新源
    pub source: String,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

// ---------- 更新源 URL 配置 ----------

const GITHUB_LATEST_JSON: &str =
    "https://github.com/AkarinLiu/smtc2web/releases/latest/download/latest.json";

/// 官网更新源 URL，编译时由 CI 环境变量 `OFFICIAL_UPDATE_URL` 注入
/// 本地开发 / CI 未设置时回退到 Cloudflare Pages 默认地址
const OFFICIAL_LATEST_JSON: &str = match option_env!("OFFICIAL_UPDATE_URL") {
    Some(url) => url,
    None => "https://smtc2web.pages.dev/latest.json",
};

/// 根据用户配置获取更新源 URL 列表（按优先级排序）
fn get_update_urls() -> Vec<String> {
    let source = {
        let app_state = APP_STATE.lock().unwrap();
        let config = app_state.config.lock().unwrap();
        config.update_source.clone()
    };

    match source.as_str() {
        "official" => vec![OFFICIAL_LATEST_JSON.to_string(), GITHUB_LATEST_JSON.to_string()],
        _ => vec![GITHUB_LATEST_JSON.to_string(), OFFICIAL_LATEST_JSON.to_string()],
    }
}

fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn get_current_platform() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("windows", "x86_64") => "windows-x86_64".to_string(),
        ("windows", "aarch64") => "windows-aarch64".to_string(),
        ("linux", "x86_64") => "linux-x86_64".to_string(),
        ("linux", "aarch64") => "linux-aarch64".to_string(),
        _ => format!("{}-{}", os, arch),
    }
}

/// 比较两个语义化版本号，返回 true 表示 `latest` > `current`
fn is_newer_version(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('-')
            .next()
            .unwrap_or("0")
            .split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };

    let c = parse(current);
    let l = parse(latest);

    for i in 0..c.len().max(l.len()) {
        let cv = c.get(i).copied().unwrap_or(0);
        let lv = l.get(i).copied().unwrap_or(0);
        match lv.cmp(&cv) {
            std::cmp::Ordering::Greater => return true,
            std::cmp::Ordering::Less => return false,
            std::cmp::Ordering::Equal => continue,
        }
    }
    false
}

async fn fetch_manifest(url: &str) -> Result<UpdateManifest, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(format!("smtc2web/{}", get_current_version()))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let manifest: UpdateManifest = resp
        .json()
        .await
        .map_err(|e| format!("解析更新清单失败: {}", e))?;

    Ok(manifest)
}

/// 核心更新检查逻辑
pub async fn check_update_inner(_app: AppHandle) -> UpdateCheckResult {
    let current = get_current_version();
    let platform = get_current_platform();
    let urls = get_update_urls();
    let main_source = {
        let app_state = APP_STATE.lock().unwrap();
        let config = app_state.config.lock().unwrap();
        config.update_source.clone()
    };

    log_info!(
        "检查更新: 当前版本 {}, 平台 {}, 首选源 {}",
        current,
        platform,
        main_source
    );

    for url in &urls {
        log_info!("尝试获取更新清单: {}", url);

        match fetch_manifest(url).await {
            Ok(manifest) => {
                if !is_newer_version(&current, &manifest.version) {
                    log_info!(
                        "已是最新版本 ({} >= {}), 源: {}",
                        current,
                        manifest.version,
                        url
                    );
                    return UpdateCheckResult {
                        has_update: false,
                        current_version: current,
                        latest_version: manifest.version,
                        notes: None,
                        download_url: None,
                        source: main_source,
                        error: None,
                    };
                }

                // 查找当前平台对应的更新包
                if let Some(platform_update) = manifest.platforms.get(&platform) {
                    log_info!(
                        "发现新版本: {} -> {}, 源: {}",
                        current,
                        manifest.version,
                        url
                    );
                    return UpdateCheckResult {
                        has_update: true,
                        current_version: current,
                        latest_version: manifest.version,
                        notes: manifest.notes.clone(),
                        download_url: Some(platform_update.url.clone()),
                        source: main_source,
                        error: None,
                    };
                } else {
                    log_info!(
                        "版本 {} 存在但无平台 {} 的包, 源: {}",
                        manifest.version,
                        platform,
                        url
                    );
                    // 继续尝试下一个源
                    continue;
                }
            }
            Err(e) => {
                log_error!("获取更新清单失败 ({}): {}", url, e);
                // 继续尝试下一个源
                continue;
            }
        }
    }

    UpdateCheckResult {
        has_update: false,
        current_version: current,
        latest_version: String::new(),
        notes: None,
        download_url: None,
        source: main_source,
        error: Some("所有更新源均不可用，请稍后重试".to_string()),
    }
}

// ---------- Tauri 命令 ----------

/// 手动检查更新
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<UpdateCheckResult, String> {
    let result = check_update_inner(app).await;
    Ok(result)
}

/// 获取可用的更新源 URL 列表（供前端显示）
#[tauri::command]
pub async fn get_update_source_urls() -> Result<Vec<(String, String)>, String> {
    Ok(vec![
        ("github".to_string(), GITHUB_LATEST_JSON.to_string()),
        ("official".to_string(), OFFICIAL_LATEST_JSON.to_string()),
    ])
}

/// 从 URL 下载更新包并执行安装
#[tauri::command]
pub async fn start_update(app: AppHandle, download_url: String) -> Result<(), String> {
    log_info!("开始下载更新: {}", download_url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .user_agent(format!("smtc2web/{}", get_current_version()))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取下载数据失败: {}", e))?;

    let temp_dir = std::env::temp_dir().join("smtc2web-update");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    let ext = if download_url.ends_with(".msi") {
        ".msi"
    } else {
        ".exe"
    };
    let installer_path = temp_dir.join(format!("update{}", ext));
    std::fs::write(&installer_path, &bytes)
        .map_err(|e| format!("写入安装包失败: {}", e))?;

    log_info!("更新包已下载到: {:?}", installer_path);

    // Windows: 使用 NSIS 或 MSI 安装器
    #[cfg(target_os = "windows")]
    {
        let path_str = installer_path.to_string_lossy().to_string();
        std::thread::spawn(move || {
            let _ = std::process::Command::new(&path_str)
                .arg("/SILENT")
                .arg("/UPDATE")
                .spawn();
        });
    }

    // Linux: 使用 deb/rpm/AppImage
    #[cfg(target_os = "linux")]
    {
        let path_str = installer_path.to_string_lossy().to_string();
        std::thread::spawn(move || {
            if download_url.ends_with(".deb") {
                let _ = std::process::Command::new("pkexec")
                    .arg("dpkg")
                    .arg("-i")
                    .arg(&path_str)
                    .spawn();
            } else if download_url.ends_with(".rpm") {
                let _ = std::process::Command::new("pkexec")
                    .arg("rpm")
                    .arg("-Uvh")
                    .arg(&path_str)
                    .spawn();
            } else {
                // AppImage: 标记为可执行并运行
                #[allow(unused_imports)]
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(
                    &path_str,
                    std::fs::Permissions::from_mode(0o755),
                );
                let _ = std::process::Command::new(&path_str)
                    .arg("--updated")
                    .spawn();
            }
        });
    }

    // 给安装器一点时间启动，然后退出当前应用
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    app.exit(0);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.4.0", "0.4.1"));
        assert!(is_newer_version("0.4.1", "1.0.0"));
        assert!(is_newer_version("1.0.0", "1.0.1"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
        assert!(!is_newer_version("2.0.0", "1.9.9"));
        assert!(is_newer_version("0.4.1", "v0.5.0"));
    }
}
