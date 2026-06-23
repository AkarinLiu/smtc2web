use clap::FromArgMatches;
use smtc2web_lib::cli::{self, Cli, Commands};

fn main() {
    let matches = cli::localized_command().get_matches();
    let cli = Cli::from_arg_matches(&matches).expect("Invalid CLI args");

    // GUI 模式：立即分离控制台（FreeConsole 瞬间完成，无闪烁）
    // dev 模式：保留控制台，输出正常可见
    if cli.command.is_none() {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::System::Console::FreeConsole;
            let _ = FreeConsole();
        }
    }

    match cli.command {
        Some(Commands::Dev(args)) => {
            let runtime = tokio::runtime::Runtime::new()
                .expect("Failed to create Tokio runtime");
            runtime.block_on(smtc2web_lib::dev::run(args));
        }
        None => {
            smtc2web_lib::run();
        }
    }
}

// 这个文件只包含入口点，所有逻辑都移到了 lib.rs 中
