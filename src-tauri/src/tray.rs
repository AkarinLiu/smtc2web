use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::process::Command;
use crate::config::Config;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend"]
#[allow(dead_code)] // 这个结构体在当前版本中未被使用
struct Asset;

pub struct TrayManager {
    should_exit: Arc<AtomicBool>,
    port: u16,
}

impl TrayManager {
    pub fn new(port: u16) -> Self {
        Self {
            should_exit: Arc::new(AtomicBool::new(false)),
            port,
        }
    }

    pub fn start(&self) {
        let should_exit = self.should_exit.clone();
        let port = self.port;

        std::thread::spawn(move || {
            Self::run_tray(should_exit, port);
        });
    }

    #[allow(dead_code)] // 这个方法在当前版本中未被使用
    pub fn should_exit(&self) -> bool {
        self.should_exit.load(Ordering::Relaxed)
    }

    fn run_tray(should_exit: Arc<AtomicBool>, port: u16) {
        use systray::Application;

        let mut app = Application::new().expect("Failed to create tray application");
        
        // // 设置托盘图标
        // let icon_bytes = Asset::get("../frontend/icon.png").expect("Failed to load icon");
        // eprintln!("Icon loaded successfully, size: {} bytes", icon_bytes.data.len());
        
        // // 尝试设置图标，如果失败则使用错误处理而不是 panic
        // if let Err(e) = app.set_icon_from_buffer(&icon_bytes.data, 32, 32) {
        //     eprintln!("Failed to set icon from buffer: {}", e);
        //     // 尝试使用内置的默认图标或者跳过图标设置
        //     // 这里我们选择跳过图标设置，让程序继续运行
        //     eprintln!("Continuing without tray icon...");
        // }
        
        // 设置窗口标题
        app.set_tooltip(&"smtc2web").expect("Failed to set tooltip");

        // 添加菜单项
        let should_exit_clone = should_exit.clone();
        app.add_menu_item("打开网页", move |_| -> Result<(), std::io::Error> {
            Self::open_web_page(port);
            Ok(())
        }).expect("Failed to add menu item");

        app.add_menu_item("打开配置文件", move |_| -> Result<(), std::io::Error> {
            Self::open_config_file();
            Ok(())
        }).expect("Failed to add menu item");

        app.add_menu_separator().expect("Failed to add separator");

        app.add_menu_item("重启应用", move |_| -> Result<(), std::io::Error> {
            Self::restart_app();
            Ok(())
        }).expect("Failed to add menu item");

        app.add_menu_item("退出应用", move |_| -> Result<(), std::io::Error> {
            should_exit_clone.store(true, Ordering::Relaxed);
            std::process::exit(0);
        }).expect("Failed to add menu item");

        // 显示托盘图标
        app.wait_for_message().expect("Failed to wait for message");
    }

    fn open_web_page(port: u16) {
        let url = format!("http://localhost:{}", port);
        if let Err(e) = open::that(&url) {
            eprintln!("Failed to open web page: {}", e);
        }
    }

    fn open_config_file() {
        let config_path = Config::get_config_path();
        if let Err(e) = open::that(config_path) {
            eprintln!("Failed to open config file: {}", e);
        }
    }

    fn restart_app() {
        let current_exe = std::env::current_exe().expect("Failed to get current executable path");
        let mut command = Command::new(current_exe);
        command.arg("--restarted");

        if let Err(e) = command.spawn() {
            eprintln!("Failed to restart application: {}", e);
        } else {
            std::process::exit(0);
        }
    }
}
