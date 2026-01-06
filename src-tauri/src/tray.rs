use tauri::tray::TrayIconBuilder;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::{AppHandle, Runtime};
use std::process;
use std::env;

/// 创建托盘菜单
pub fn create_tray_menu<R: Runtime>(app: &AppHandle<R>) -> Menu<R> {
    let open_web = MenuItem::with_id(app, "open_web", "打开网页", true, None::<&str>).unwrap();
    let open_config = MenuItem::with_id(app, "open_config", "打开配置文件", true, None::<&str>).unwrap();
    let restart = MenuItem::with_id(app, "restart", "重启应用", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "退出应用", true, None::<&str>).unwrap();

    Menu::with_items(app, &[
        &open_web,
        &open_config,
        &PredefinedMenuItem::separator(app).unwrap(),
        &restart,
        &quit,
    ]).unwrap()
}



/// 处理托盘菜单事件
pub fn handle_tray_menu_event<R: Runtime>(_app: &AppHandle<R>, event: tauri::menu::MenuEvent, port: u16) {
    match event.id.as_ref() {
        "open_web" => {
            let url = format!("http://localhost:{}", port);
            if let Err(e) = open::that(&url) {
                eprintln!("Failed to open web page: {}", e);
            }
        }
        "open_config" => {
            let config_path = crate::config::Config::get_config_path();
            if let Err(e) = open::that(config_path) {
                eprintln!("Failed to open config file: {}", e);
            }
        }
        "restart" => {
            let current_exe = env::current_exe().expect("Failed to get current executable path");
            let mut command = std::process::Command::new(current_exe);
            command.arg("--restarted");

            if let Err(e) = command.spawn() {
                eprintln!("Failed to restart application: {}", e);
            } else {
                process::exit(0);
            }
        }
        "quit" => {
            process::exit(0);
        }
        _ => {}
    }
}

/// 创建系统托盘图标并配置事件处理
pub fn create_tray_icon<R: Runtime>(
    app: &AppHandle<R>,
    port: u16,
) -> Result<(), tauri::Error> {
    // 创建托盘菜单
    let tray_menu = create_tray_menu(app);
    
    // 创建系统托盘图标
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&tray_menu)
        .show_menu_on_left_click(true) // 启用左键点击弹出菜单
        .on_menu_event(move |app, event| {
            handle_tray_menu_event(app, event, port);
        })
        .build(app)?;
    
    Ok(())
}