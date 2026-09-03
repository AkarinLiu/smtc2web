use clap::{Args, Parser, Subcommand};
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "smtc2web", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// (placeholder - actual about set at runtime)
    Dev(DevArgs),
}

#[derive(Args, Clone)]
pub struct DevArgs {
    /// (placeholder)
    #[arg(short = 'P', long, default_value = "3031")]
    pub port: u16,

    /// (placeholder)
    #[arg(long)]
    pub no_open: bool,

    /// (placeholder)
    #[arg(long)]
    pub vite: bool,

    /// (placeholder)
    #[arg(long, default_value = "5173")]
    pub vite_port: u16,

    /// (placeholder)
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

/* ======================== Embedded locale files ======================== */

#[derive(RustEmbed)]
#[folder = "locales"]
struct CliLocales;

#[derive(Debug, Deserialize)]
struct CliDevArgs {
    port: String,
    #[serde(rename = "no-open")]
    no_open: String,
    vite: String,
    #[serde(rename = "vite-port")]
    vite_port: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct CliDev {
    about: String,
    args: CliDevArgs,
}

#[derive(Debug, Deserialize)]
struct CliRoot {
    about: String,
    dev: CliDev,
}

#[derive(Debug, Deserialize)]
struct LocaleData {
    cli: CliRoot,
}

fn load_locale(lang: &str) -> LocaleData {
    let filename = format!("{}.toml", lang);
    let fallback = "en.toml";

    let content = CliLocales::get(&filename)
        .or_else(|| CliLocales::get(fallback))
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .unwrap_or_default();

    toml::from_str(&content).unwrap_or_else(|_| {
        let fb = CliLocales::get(fallback)
            .map(|f| String::from_utf8_lossy(&f.data).to_string())
            .unwrap_or_default();
        toml::from_str(&fb).unwrap()
    })
}

/* ======================== Language detection ======================== */

fn detect_system_lang() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Globalization::GetUserDefaultUILanguage;
        // SAFETY: GetUserDefaultUILanguage is a trivial Win32 FFI call — no preconditions, returns a LANGID.
        let lang_id = unsafe { GetUserDefaultUILanguage() };
        match lang_id {
            0x0804 | 0x0c04 | 0x1004 | 0x1404 => return "zh-CN",
            0x0404 => return "zh-TW",
            0x0407 | 0x0807 | 0x0c07 | 0x1007 | 0x1407 => return "de-DE",
            0x040c | 0x080c | 0x0c0c | 0x100c => return "fr-FR",
            0x0410 | 0x0810 => return "it-IT",
            0x0411 => return "ja-JP",
            0x0412 => return "ko-KR",
            0x0413 | 0x0813 => return "nl-NL",
            0x0419 => return "ru-RU",
            _ => {}
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        for var in &["LANG", "LC_ALL", "LC_MESSAGES"] {
            if let Ok(val) = std::env::var(var) {
                let lo = val.to_lowercase();
                if lo.starts_with("zh_cn") || lo.starts_with("zh-cn") || lo.starts_with("zh_sg") {
                    return "zh-CN";
                }
                if lo.starts_with("zh_tw") || lo.starts_with("zh-tw") || lo.starts_with("zh_hk") {
                    return "zh-TW";
                }
                if lo.starts_with("zh") {
                    return "zh-CN";
                }
                if lo.starts_with("de_") || lo.starts_with("de-") {
                    return "de-DE";
                }
                if lo.starts_with("fr_") || lo.starts_with("fr-") {
                    return "fr-FR";
                }
                if lo.starts_with("it_") || lo.starts_with("it-") {
                    return "it-IT";
                }
                if lo.starts_with("ja_") || lo.starts_with("ja-") {
                    return "ja-JP";
                }
                if lo.starts_with("ko_kp") || lo.starts_with("ko-kp") {
                    return "ko-KP";
                }
                if lo.starts_with("ko_") || lo.starts_with("ko-") {
                    return "ko-KR";
                }
                if lo.starts_with("nl_") || lo.starts_with("nl-") {
                    return "nl-NL";
                }
                if lo.starts_with("ru_") || lo.starts_with("ru-") {
                    return "ru-RU";
                }
            }
        }
    }

    "en"
}

/* ======================== Build localized Command ======================== */

pub fn localized_command() -> clap::Command {
    use clap::CommandFactory;

    let lang = detect_system_lang();
    let l10n = load_locale(lang);

    Cli::command()
        .about(l10n.cli.about)
        .mut_subcommand("dev", |cmd| {
            cmd.about(l10n.cli.dev.about)
                .mut_arg("port", |a| a.help(l10n.cli.dev.args.port))
                .mut_arg("no_open", |a| a.help(l10n.cli.dev.args.no_open))
                .mut_arg("vite", |a| a.help(l10n.cli.dev.args.vite))
                .mut_arg("vite_port", |a| a.help(l10n.cli.dev.args.vite_port))
                .mut_arg("path", |a| a.help(l10n.cli.dev.args.path))
        })
}
