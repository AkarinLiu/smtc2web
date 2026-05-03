#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::*;

#[cfg(target_os = "windows")]
pub fn hide_console() {
    use windows::Win32::System::Console::GetConsoleWindow;
    unsafe {
        let console_window = GetConsoleWindow();
        if !console_window.is_invalid() {
            let _ = ShowWindow(console_window, SW_HIDE);
        }
    }
}

#[cfg(target_os = "linux")]
pub fn hide_console() {
    // Linux has no console window concept — no-op
}
