# System Integration Skill

## Single Instance

### Implementation
使用 `named-lock` crate 实现单实例:

```rust
use named_lock::NamedLock;

fn check_single_instance() -> Result<NamedLock, String> {
    let lock = NamedLock::create("app_name_single_instance_mutex")
        .map_err(|_| "创建命名锁失败".to_string())?;

    if let Err(_) = lock.try_lock() {
        // 锁已被占用，打开已有实例
        let url = format!("http://localhost:{}", port);
        let _ = that(&url);
        return Err("程序已在运行".to_string());
    }

    Ok(lock)
}
```

### Usage
```rust
let _mutex_handle = match check_single_instance() {
    Ok(handle) => handle,
    Err(e) => {
        eprintln!("{}", e);
        std::process::exit(1);
    }
};
```

## System Tray

### Create Tray Icon
```rust
use tauri::tray::TrayIconBuilder;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

pub fn create_tray_icon<R: Runtime>(app: &AppHandle<R>, port: u16) -> Result<(), tauri::Error> {
    let menu = create_tray_menu(app);
    
    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| {
            handle_tray_menu_event(app, event, port);
        })
        .build(app)?;
    
    Ok(())
}
```

### Tray Menu Items
```rust
pub fn create_tray_menu<R: Runtime>(app: &AppHandle<R>) -> Menu<R> {
    let open_web = MenuItem::with_id(app, "open_web", "打开网页", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap();

    Menu::with_items(app, &[&open_web, &quit]).unwrap()
}
```

### Handle Menu Events
```rust
pub fn handle_tray_menu_event<R: Runtime>(_app: &AppHandle<R>, event: tauri::menu::MenuEvent, port: u16) {
    match event.id.as_ref() {
        "open_web" => {
            let url = format!("http://localhost:{}", port);
            let _ = that(&url);
        }
        "quit" => {
            std::process::exit(0);
        }
        _ => {}
    }
}
```

## Configuration

### Config Location
`%APPDATA%/smtc2web/config.toml`

### Default Config
```rust
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
```

### Hot Reload
```rust
Config::start_monitoring(config.clone());
// 配置文件修改后自动重新加载
```

## Restart Application

```rust
use std::env;

fn restart_app() {
    let current_exe = env::current_exe().expect("Failed to get executable path");
    let mut command = std::process::Command::new(current_exe);
    command.arg("--restarted");
    
    let _ = command.spawn();
    std::process::exit(0);
}
```

## Theme/File Serving

### Serve Custom Files
```rust
let theme_manager = theme::ThemeManager::new(&theme_path);

let static_files = warp::path::tail()
    .and(theme::ThemeManager::with_manager(theme_manager))
    .and_then(|tail, manager| manager.serve_theme_file(tail));
```
