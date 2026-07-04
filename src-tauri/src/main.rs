// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use clap::FromArgMatches;
use smtc2web_lib::cli::{self, Cli, Commands};

fn main() {
    let matches = cli::localized_command().get_matches();
    let cli = Cli::from_arg_matches(&matches).expect("Invalid CLI args");

    match cli.command {
        Some(Commands::Dev(args)) => {
            // On Windows release builds, attach to the parent console or allocate one
            #[cfg(all(not(debug_assertions), target_os = "windows"))]
            unsafe {
                use windows::Win32::System::Console::AttachConsole;
                // ATTACH_PARENT_CONSOLE = 0xFFFFFFFF
                if AttachConsole(0xFFFFFFFF).is_err() {
                    // No parent console, allocate our own
                    use windows::Win32::System::Console::AllocConsole;
                    let _ = AllocConsole();
                }
            }

            let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            runtime.block_on(smtc2web_lib::dev::run(args));
        }
        None => {
            // GUI 模式：立即分离控制台（仅 Windows）
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::System::Console::FreeConsole;
                let _ = FreeConsole();
            }

            smtc2web_lib::run();
        }
    }
}
