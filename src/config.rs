use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use dirs::config_dir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub server_port: u16,
    pub show_console: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_port: 3030,
            show_console: false,
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

    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        
        if !config_path.exists() {
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent).unwrap_or_else(|_| {
                    eprintln!("Failed to create config directory");
                });
            }
            
            let default_config = Self::default();
            if let Err(e) = default_config.save() {
                eprintln!("Failed to save default config: {}", e);
            }
            return default_config;
        }

        let content = match fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to read config file: {}", e);
                return Self::default();
            }
        };

        match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to parse config file: {}", e);
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }
}
