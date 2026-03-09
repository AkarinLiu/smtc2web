use chrono::Local;
use dirs::data_dir;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// 日志管理器
pub struct Logger {
    log_dir: PathBuf,
    current_level: LogLevel,
    max_age_days: i64,
}

impl Logger {
    /// 获取单例实例
    pub fn instance() -> &'static Mutex<Logger> {
        static INSTANCE: once_cell::sync::Lazy<Mutex<Logger>> =
            once_cell::sync::Lazy::new(|| Mutex::new(Logger::new()));
        &INSTANCE
    }

    /// 创建新的日志管理器
    fn new() -> Self {
        let log_dir = Self::get_log_dir();

        // 确保日志目录存在
        if let Err(e) = fs::create_dir_all(&log_dir) {
            eprintln!("创建日志目录失败: {}", e);
        }

        let logger = Self {
            log_dir,
            current_level: LogLevel::Info,
            max_age_days: 30,
        };

        // 清理旧日志
        logger.clean_old_logs();

        logger
    }

    /// 获取日志目录路径
    fn get_log_dir() -> PathBuf {
        let mut dir = data_dir().unwrap_or_else(|| PathBuf::from("."));
        dir.push("smtc2web");
        dir.push("log");
        dir
    }

    /// 获取当前日期的日志文件路径
    fn get_current_log_file(&self) -> PathBuf {
        let date = Local::now().format("%Y-%m-%d").to_string();
        self.log_dir.join(format!("{}.log", date))
    }

    /// 设置日志级别
    pub fn set_level(&mut self, level: LogLevel) {
        self.current_level = level;
    }

    /// 设置日志保留天数
    pub fn set_max_age_days(&mut self, days: i64) {
        self.max_age_days = days;
    }

    /// 写入日志
    pub fn log(&self, level: LogLevel, message: &str) {
        // 只记录达到当前级别或更高级别的日志
        if level < self.current_level {
            return;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        let log_entry = format!("[{}] [{}] {}\n", timestamp, level.as_str(), message);

        let log_file = self.get_current_log_file();

        // 使用追加模式打开文件
        match fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(log_entry.as_bytes()) {
                    eprintln!("写入日志失败: {}", e);
                }
            }
            Err(e) => {
                eprintln!("打开日志文件失败: {}", e);
            }
        }
    }

    /// 清理旧日志
    fn clean_old_logs(&self) {
        let cutoff = chrono::Local::now() - chrono::Duration::days(self.max_age_days);

        match fs::read_dir(&self.log_dir) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();

                        // 只处理 .log 文件
                        if path.extension().and_then(|s| s.to_str()) != Some("log") {
                            continue;
                        }

                        // 尝试从文件名解析日期
                        if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                            if let Ok(file_date) =
                                chrono::NaiveDate::parse_from_str(filename, "%Y-%m-%d")
                            {
                                let file_datetime = chrono::NaiveDateTime::new(
                                    file_date,
                                    chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap_or_default(),
                                );

                                if let Some(cutoff_naive) = cutoff.naive_local().into() {
                                    if file_datetime < cutoff_naive {
                                        if let Err(e) = fs::remove_file(&path) {
                                            self.log(
                                                LogLevel::Warn,
                                                &format!(
                                                    "删除旧日志文件失败: {} - {}",
                                                    path.display(),
                                                    e
                                                ),
                                            );
                                        } else {
                                            self.log(
                                                LogLevel::Info,
                                                &format!("已删除旧日志文件: {}", path.display()),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("读取日志目录失败: {}", e);
            }
        }
    }
}

/// 初始化日志系统
pub fn init() {
    Logger::instance();
}

/// 设置日志级别
pub fn set_level(level: LogLevel) {
    if let Ok(mut logger) = Logger::instance().lock() {
        logger.set_level(level);
    }
}

/// 设置日志保留天数
pub fn set_max_age_days(days: i64) {
    if let Ok(mut logger) = Logger::instance().lock() {
        logger.set_max_age_days(days);
    }
}

/// 记录调试日志
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::Logger::instance()
            .lock()
            .unwrap()
            .log($crate::logger::LogLevel::Debug, &format!($($arg)*))
    };
}

/// 记录信息日志
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::Logger::instance()
            .lock()
            .unwrap()
            .log($crate::logger::LogLevel::Info, &format!($($arg)*))
    };
}

/// 记录警告日志
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logger::Logger::instance()
            .lock()
            .unwrap()
            .log($crate::logger::LogLevel::Warn, &format!($($arg)*))
    };
}

/// 记录错误日志
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::Logger::instance()
            .lock()
            .unwrap()
            .log($crate::logger::LogLevel::Error, &format!($($arg)*))
    };
}

// 宏通过 #[macro_export] 自动导出，无需再次 pub use
