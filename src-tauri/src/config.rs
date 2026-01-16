use dirs::config_dir;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config as NotifyConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;



#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub server_port: u16,
    pub show_console: bool,
    pub address: String,
    pub theme_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_port: 3030,
            show_console: false,
            address: "127.0.0.1".to_string(),
            theme_path: "".to_string(),
        }
    }
}

impl Config {
    pub fn get_config_path() -> PathBuf {
        let mut config_path = config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_path.push("smtc2web");
        config_path.push("config.toml");
        config_path
    }

    // 已被下面的带错误处理的load方法替代

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 使用默认的 TOML 序列化
        let content = toml::to_string_pretty(self)?;

        fs::write(&config_path, content)?;
        Ok(())
    }

    /// 启动配置文件监控
    pub fn start_monitoring(config: Arc<Mutex<Self>>) {
        let config_path = Self::get_config_path();
        let config_dir = config_path.parent().unwrap_or_else(|| config_path.as_path());

        // 创建监视器
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher: RecommendedWatcher = Watcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            NotifyConfig::default()
        ).expect("Failed to create watcher");

        // 监听配置文件
        if let Err(e) = watcher.watch(config_dir, RecursiveMode::NonRecursive) {
            eprintln!("Failed to watch config directory: {}", e);
            return;
        }

        // 在单独的线程中处理文件变化事件
        thread::spawn(move || {
            let mut last_modified = fs::metadata(&config_path)
                .ok()
                .and_then(|meta| meta.modified().ok());

            loop {
                match rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(Ok(event)) => {
                        // 检查是否是配置文件的修改事件
                        if event.paths.contains(&config_path) {
                            // 避免重复处理同一修改事件
                            let current_modified = fs::metadata(&config_path)
                                .ok()
                                .and_then(|meta| meta.modified().ok());

                            if current_modified != last_modified {
                                last_modified = current_modified;

                                // 重新加载配置
                                match Self::load() {
                                    Ok(new_config) => {
                                        println!("Config file updated, reloading...");
                                        // 更新共享配置
                                        let mut config_guard = config.lock().unwrap();
                                        *config_guard = new_config;
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to reload config: {}", e);
                                    }
                                }
                            }
                        }
                    },
                    Ok(Err(e)) => {
                        eprintln!("Watch error: {}", e);
                    },
                    Err(_) => {
                        // 超时，继续监听
                    },
                }
            }
        });
    }

    /// 加载配置并处理错误
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();

        if !config_path.exists() {
            // 创建配置目录
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // 保存默认配置并返回
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        // 读取配置文件并解析
        let content = fs::read_to_string(&config_path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
