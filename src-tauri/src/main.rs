fn main() {
    #[cfg(feature = "dev")]
    {
        use clap::FromArgMatches;
        use smtc2web_lib::cli::{self, Cli, Commands};

        let matches = cli::localized_command().get_matches();
        let cli = Cli::from_arg_matches(&matches).expect("Invalid CLI args");

        if cli.command.is_none() {
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::System::Console::FreeConsole;
                let _ = FreeConsole();
            }
        }

        match cli.command {
            Some(Commands::Dev(args)) => {
                let runtime =
                    tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
                runtime.block_on(smtc2web_lib::dev::run(args));
            }
            None => {
                smtc2web_lib::run();
            }
        }
        return;
    }

    #[cfg(not(feature = "dev"))]
    smtc2web_lib::run();
}
