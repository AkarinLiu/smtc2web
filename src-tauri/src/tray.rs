use open::that;
use std::process;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

/// 创建托盘菜单
pub fn create_tray_menu<R: Runtime>(app: &AppHandle<R>) -> Menu<R> {
    let show_window =
        MenuItem::with_id(app, "show_window", "显示窗口", true, None::<&str>).unwrap();
    let open_web = MenuItem::with_id(app, "open_web", "打开网页", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "退出应用", true, None::<&str>).unwrap();

    Menu::with_items(
        app,
        &[
            &show_window,
            &PredefinedMenuItem::separator(app).unwrap(),
            &open_web,
            &PredefinedMenuItem::separator(app).unwrap(),
            &quit,
        ],
    )
    .unwrap()
}

/// 显示窗口
fn show_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _: Result<(), _> = window.show();
        let _: Result<(), _> = window.set_focus();
    }
}

/// 处理托盘菜单事件
pub fn handle_tray_menu_event<R: Runtime>(
    app: &AppHandle<R>,
    event: tauri::menu::MenuEvent,
    port: u16,
) {
    match event.id.as_ref() {
        "show_window" => {
            show_window(app);
        }
        "open_web" => {
            let url = format!("http://localhost:{}", port);
            if let Err(e) = that(&url) {
                eprintln!("Failed to open web page: {}", e);
            }
        }
        "quit" => {
            process::exit(0);
        }
        _ => {}
    }
}

/// 处理托盘图标事件（双击显示窗口）
fn handle_tray_icon_event<R: Runtime>(app: &AppHandle<R>, event: TrayIconEvent) {
    match event {
        TrayIconEvent::DoubleClick { .. } => {
            show_window(app);
        }
        _ => {}
    }
}

/// 创建系统托盘图标并配置事件处理
pub fn create_tray_icon<R: Runtime>(app: &AppHandle<R>, port: u16) -> Result<(), tauri::Error> {
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
        .on_tray_icon_event(move |tray, event| {
            handle_tray_icon_event(tray.app_handle(), event);
        })
        .build(app)?;

    Ok(())
}
