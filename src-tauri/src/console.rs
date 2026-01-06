use windows::Win32::System::Console::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub fn hide_console() {
    unsafe {
        let console_window = GetConsoleWindow();
        if !console_window.is_invalid() {
            let _ = ShowWindow(console_window, SW_HIDE);
        }
    }
}
