use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static ALBUM_ART_CACHE: Lazy<Mutex<HashMap<String, (String, u64)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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
        entries.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));
        let to_remove: Vec<String> = entries.iter().skip(30).map(|(k, _)| (*k).clone()).collect();
        for key in to_remove {
            cache.remove(key.as_str());
        }
    }
}

#[cfg(target_os = "windows")]
mod smtc;
#[cfg(target_os = "windows")]
pub type PlatformSession = smtc::SmtcSession;

#[cfg(target_os = "linux")]
mod mpris;
#[cfg(target_os = "linux")]
pub type PlatformSession = mpris::MprisSession;
