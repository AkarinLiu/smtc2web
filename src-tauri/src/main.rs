// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    smtc2web_lib::run()
}

// 这个文件只包含入口点，所有逻辑都移到了 lib.rs 中
