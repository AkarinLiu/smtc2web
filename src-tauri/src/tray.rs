use crate::i18n::{get_current_locale_data, set_locale};
use crate::{log_info, log_warn};
use std::process;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

/// 全局端口存储，用于重建菜单
static TRAY_PORT: once_cell::sync::Lazy<Mutex<u16>> =
    once_cell::sync::Lazy::new(|| Mutex::new(3030));

/// 创建托盘菜单（根据当前语言）
pub fn create_tray_menu<R: Runtime>(app: &AppHandle<R>) -> Menu<R> {
    // 获取当前语言的翻译
    let translations = get_current_locale_data()
        .map(|l| l.tray)
        .unwrap_or_else(|| crate::i18n::TrayTranslations {
            show_window: "显示窗口".to_string(),
            open_web: "打开网页".to_string(),
            quit: "退出应用".to_string(),
        });

    let show_window = MenuItem::with_id(
        app,
        "show_window",
        translations.show_window,
        true,
        None::<&str>,
    )
    .unwrap();
    let open_web =
        MenuItem::with_id(app, "open_web", translations.open_web, true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", translations.quit, true, None::<&str>).unwrap();

    Menu::with_items(app, &[&show_window, &open_web, &quit]).unwrap()
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
            if let Err(e) = open::that(&url) {
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
    if let TrayIconEvent::DoubleClick { .. } = event {
        show_window(app);
    }
}

/// 创建系统托盘图标并配置事件处理
/// 使用泛型保持与原始代码的兼容性
pub fn create_tray_icon<R: Runtime>(app: &AppHandle<R>, port: u16) -> Result<(), tauri::Error> {
    // 存储端口
    {
        let mut port_guard = TRAY_PORT.lock().unwrap();
        *port_guard = port;
    }

    // 创建托盘菜单
    let tray_menu = create_tray_menu(app);

    // 创建系统托盘图标
    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&tray_menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| {
            let port = *TRAY_PORT.lock().unwrap();
            handle_tray_menu_event(app, event, port);
        })
        .on_tray_icon_event(move |tray, event| {
            handle_tray_icon_event(tray.app_handle(), event);
        })
        .build(app)?;

    Ok(())
}

/// 更新托盘菜单语言
/// 实时更新托盘菜单显示语言
pub fn update_tray_menu_language<R: Runtime>(
    app: &AppHandle<R>,
    new_locale: &str,
) -> Result<(), String> {
    // 设置新语言
    set_locale(new_locale)?;

    // 获取托盘图标并更新菜单
    // 通过 app.tray_by_id 获取托盘图标
    if let Some(tray) = app.tray_by_id("main-tray") {
        // 创建新语言的菜单
        let new_menu = create_tray_menu(app);
        // 更新托盘菜单
        if let Err(e) = tray.set_menu(Some(new_menu)) {
            log_warn!("Failed to update tray menu: {}", e);
            return Err(format!("Failed to update tray menu: {}", e));
        }
        log_info!("Tray menu language updated to: {}", new_locale);
    } else {
        log_warn!("Tray icon not found, language will apply on next restart");
    }

    log_info!("Locale set to: {}", new_locale);

    Ok(())
}
