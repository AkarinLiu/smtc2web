use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

pub fn list_system_fonts() -> Vec<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();

    for dir in system_font_dirs() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let ext = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if ext != "ttf" && ext != "otf" && ext != "ttc" {
                    continue;
                }
                if let Ok(data) = fs::read(&path) {
                    if let Some(name) = parse_font_family(&data) {
                        names.insert(name);
                    }
                    if ext == "ttc"
                        && let Some(collect) = parse_ttc_families(&data)
                    {
                        names.extend(collect);
                    }
                }
            }
        }
    }

    names.into_iter().collect()
}

fn parse_font_family(data: &[u8]) -> Option<String> {
    let face = ttf_parser::Face::parse(data, 0).ok()?;
    for name in face.names() {
        if name.name_id == 1
            && let Some(s) = name.to_string()
        {
            return Some(s);
        }
    }
    None
}

fn parse_ttc_families(data: &[u8]) -> Option<Vec<String>> {
    let count = ttf_parser::fonts_in_collection(data)?;
    let mut names = Vec::with_capacity(count as usize);
    for i in 0..count {
        if let Ok(face) = ttf_parser::Face::parse(data, i) {
            for name in face.names() {
                if name.name_id == 1
                    && let Some(s) = name.to_string()
                {
                    names.push(s);
                    break;
                }
            }
        }
    }
    if names.is_empty() { None } else { Some(names) }
}

fn system_font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if let Ok(windir) = std::env::var("WINDIR") {
            dirs.push(PathBuf::from(windir).join("Fonts"));
        }
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            dirs.push(
                PathBuf::from(local)
                    .join("Microsoft")
                    .join("Windows")
                    .join("Fonts"),
            );
        }
    }

    #[cfg(target_os = "macos")]
    {
        dirs.push(PathBuf::from("/System/Library/Fonts"));
        dirs.push(PathBuf::from("/Library/Fonts"));
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(home).join("Library").join("Fonts"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        dirs.push(PathBuf::from("/usr/share/fonts"));
        dirs.push(PathBuf::from("/usr/local/share/fonts"));
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(&home).join(".fonts"));
            dirs.push(
                PathBuf::from(&home)
                    .join(".local")
                    .join("share")
                    .join("fonts"),
            );
        }
    }

    dirs
}
