use std::collections::HashMap;
use std::sync::Mutex;

static ALBUM_ART_CACHE: std::sync::LazyLock<Mutex<HashMap<String, (String, u64)>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Default)]
pub struct SessionInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub is_playing: bool,
    pub position_secs: u64,
    pub duration_secs: u64,
    pub app_id: String,
    pub app_name: String,
}

#[cfg(target_os = "linux")]
pub trait MediaSession: Send + 'static {
    fn new(process_filter: &str) -> Result<Self, String>
    where
        Self: Sized;
    fn poll_current(&self) -> Option<SessionInfo>;
    fn get_album_art_base64(&self, artist: &str, title: &str, album: &str) -> Option<String>;
}

pub(crate) fn generate_song_id(title: &str, artist: &str, album: &str) -> String {
    format!("{}|{}|{}", title, artist, album)
}

pub(crate) fn matches_process_filter(filter: &str, id: &str, name: &str) -> bool {
    let filter = filter.trim();
    if filter == "*" || filter.is_empty() {
        return true;
    }
    let id_lower = id.to_lowercase();
    let name_lower = name.to_lowercase();
    filter.lines().any(|line| {
        let pattern = line.trim().to_lowercase();
        if pattern.is_empty() {
            return false;
        }
        id_lower.contains(&pattern) || name_lower.contains(&pattern)
    })
}

pub(crate) fn get_cached_album_art(song_id: &str) -> Option<String> {
    let cache = ALBUM_ART_CACHE.lock().unwrap();
    cache.get(song_id).map(|(art, _)| art.clone())
}

pub(crate) fn set_cached_album_art(song_id: &str, art: String) {
    let mut cache = ALBUM_ART_CACHE.lock().unwrap();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    cache.insert(song_id.to_string(), (art, timestamp));

    if cache.len() > 30 {
        let mut entries: Vec<_> = cache.iter().collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.1.1));
        let to_remove: Vec<String> = entries.iter().skip(30).map(|(k, _)| (*k).clone()).collect();
        for key in to_remove {
            cache.remove(key.as_str());
        }
    }
}

#[cfg(target_os = "windows")]
pub mod smtc;

#[cfg(target_os = "linux")]
mod mpris;
#[cfg(target_os = "linux")]
pub type PlatformSession = mpris::MprisSession;

// ponytail: shared Linux polling loop — deduplicates lib.rs and dev.rs
#[cfg(target_os = "linux")]
pub(crate) fn poll_media_loop(
    session: &impl MediaSession,
    state: &crate::Shared,
    app_id_global: &std::sync::Mutex<String>,
    app_name_global: &std::sync::Mutex<String>,
) {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let mut last_song = crate::Song::default();
    let mut last_position = None::<String>;
    let mut last_song_id = String::new();
    let mut last_art_update = 0u64;

    loop {
        let mut current_song = crate::Song::default();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        current_song.last_update = timestamp;

        if let Some(info) = session.poll_current() {
            *app_id_global.lock().unwrap() = info.app_id.clone();
            *app_name_global.lock().unwrap() = info.app_name.clone();

            current_song.is_playing = info.is_playing;
            current_song.title = info.title;
            current_song.artist = info.artist;
            current_song.album = info.album;

            let current_song_id = generate_song_id(
                &current_song.title,
                &current_song.artist,
                &current_song.album,
            );
            let cached_art = get_cached_album_art(&current_song_id);

            let should_fetch_art = cached_art.is_none()
                || (current_song_id != last_song_id
                    && timestamp.saturating_sub(last_art_update) > 30);

            if should_fetch_art {
                current_song.album_art = session.get_album_art_base64(
                    &current_song.artist,
                    &current_song.title,
                    &current_song.album,
                );
                if let Some(ref art) = current_song.album_art {
                    set_cached_album_art(&current_song_id, art.clone());
                }
                last_song_id = current_song_id;
                last_art_update = timestamp;
            } else {
                current_song.album_art = cached_art;
            }

            if info.duration_secs > 0 {
                current_song.position = Some(crate::format_duration(info.position_secs));
                current_song.duration = Some(crate::format_duration(info.duration_secs));
                let pct = (info.position_secs as f64 * 100.0) / info.duration_secs as f64;
                current_song.pct = Some((pct * 10.0).round() / 10.0);
            }
        } else {
            *state.write().unwrap() = crate::Song::default();
            last_song = crate::Song::default();
            last_position = None;
            std::thread::sleep(Duration::from_millis(500));
            continue;
        }

        let should_update = current_song.is_playing != last_song.is_playing
            || current_song.position != last_position
            || current_song.title != last_song.title
            || current_song.artist != last_song.artist
            || current_song.album != last_song.album
            || current_song.album_art != last_song.album_art
            || timestamp.saturating_sub(last_song.last_update) > 10;

        if should_update {
            *state.write().unwrap() = current_song.clone();
            last_song = current_song.clone();
            last_position = current_song.position.clone();
        }

        let sleep_duration = match current_song.is_playing {
            true => Duration::from_millis(200),
            false => Duration::from_millis(1000),
        };
        std::thread::sleep(sleep_duration);
    }
}
