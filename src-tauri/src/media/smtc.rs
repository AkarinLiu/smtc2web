use super::{generate_song_id, get_cached_album_art, set_cached_album_art, MediaSession, SessionInfo};
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
use windows::Management::Deployment::PackageManager;

static AUMID_DISPLAY_NAME_CACHE: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub struct SmtcSession {
    manager: GlobalSystemMediaTransportControlsSessionManager,
    runtime: tokio::runtime::Runtime,
    process_filter: String,
}

impl SmtcSession {
    fn matches_filter(&self, app_id: &str, app_name: &str) -> bool {
        let filter = self.process_filter.trim();

        if filter == "*" || filter.is_empty() {
            return true;
        }

        let app_id_lower = app_id.to_lowercase();
        let app_name_lower = app_name.to_lowercase();

        filter.lines().any(|line| {
            let pattern = line.trim().to_lowercase();
            if pattern.is_empty() {
                return false;
            }
            app_id_lower.contains(&pattern) || app_name_lower.contains(&pattern)
        })
    }

    fn get_app_display_name(&self, aumid: &str) -> String {
        if aumid.is_empty() {
            return String::new();
        }

        {
            let cache = AUMID_DISPLAY_NAME_CACHE.lock().unwrap();
            if let Some(name) = cache.get(aumid) {
                return name.clone();
            }
        }

        let display_name = if is_store_app(aumid) {
            get_store_app_display_name(aumid)
                .unwrap_or_else(|| get_fallback_display_name(aumid))
        } else {
            get_fallback_display_name(aumid)
        };

        {
            let mut cache = AUMID_DISPLAY_NAME_CACHE.lock().unwrap();
            cache.insert(aumid.to_string(), display_name.clone());
        }

        display_name
    }
}

fn is_store_app(aumid: &str) -> bool {
    aumid.contains('!') && aumid.contains('_')
}

fn get_store_app_display_name(aumid: &str) -> Option<String> {
    let family_name = aumid.split('!').next()?;

    let package_manager = PackageManager::new().ok()?;
    let packages = package_manager.FindPackages().ok()?;

    for package in packages {
        if let Ok(id) = package.Id() {
            if let Ok(family) = id.FamilyName() {
                if family.to_string() == family_name {
                    if let Ok(display_name) = package.DisplayName() {
                        let name = display_name.to_string();
                        if !name.is_empty() && name != family_name {
                            return Some(name);
                        }
                    }
                }
            }
        }
    }

    None
}

fn get_fallback_display_name(aumid: &str) -> String {
    if aumid.len() > 20 {
        format!("{}...", &aumid[..20])
    } else {
        aumid.to_string()
    }
}

impl MediaSession for SmtcSession {
    fn new(process_filter: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .and_then(|f| f.get())
            .map_err(|e| format!("Failed to get SMTC session manager: {:?}", e))?;

        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create Tokio runtime: {}", e))?;

        Ok(SmtcSession {
            manager,
            runtime,
            process_filter: process_filter.to_string(),
        })
    }

    fn poll_current(&self) -> Option<SessionInfo> {
        let session = self.manager.GetCurrentSession().ok()?;

        let app_id = session
            .SourceAppUserModelId()
            .ok()
            .and_then(|h| Some(h.to_string()))
            .unwrap_or_default();

        let app_name = self.get_app_display_name(&app_id);

        if !self.matches_filter(&app_id, &app_name) {
            return None;
        }

        let mut info = SessionInfo {
            app_id,
            app_name,
            ..Default::default()
        };

        if let Ok(playback_info) = session.GetPlaybackInfo() {
            use windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus;
            info.is_playing = playback_info.PlaybackStatus().unwrap_or_default()
                == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
        }

        if let Ok(media_info) = session.TryGetMediaPropertiesAsync().and_then(|f| f.get()) {
            info.title = media_info.Title().unwrap_or_default().to_string();
            info.artist = media_info.Artist().unwrap_or_default().to_string();
            info.album = media_info.AlbumTitle().unwrap_or_default().to_string();
        }

        if let Ok(timeline) = session.GetTimelineProperties() {
            let pos = timeline.Position().unwrap().Duration;
            let dur = timeline.EndTime().unwrap().Duration;
            info.position_secs = (pos / 10_000_000) as u64;
            info.duration_secs = (dur / 10_000_000) as u64;
        }

        Some(info)
    }

    fn get_album_art_base64(&self, artist: &str, title: &str, album: &str) -> Option<String> {
        let song_id = generate_song_id(title, artist, album);

        if let Some(cached) = get_cached_album_art(&song_id) {
            return Some(cached);
        }

        let session = self.manager.GetCurrentSession().ok()?;

        let thumbnail_data = self.runtime.block_on(fetch_thumbnail(&session))?;

        use base64::{engine::general_purpose::STANDARD, Engine};
        let mime = "data:image/jpeg";
        let data_uri = format!("{};base64,{}", mime, STANDARD.encode(&thumbnail_data));

        set_cached_album_art(&song_id, data_uri.clone());

        Some(data_uri)
    }
}

async fn fetch_thumbnail(
    session: &windows::Media::Control::GlobalSystemMediaTransportControlsSession,
) -> Option<Vec<u8>> {
    use windows::Storage::Streams::{Buffer, DataReader, InputStreamOptions};

    let info = session
        .TryGetMediaPropertiesAsync()
        .and_then(|f| f.get())
        .ok()?;

    let thumbnail = info.Thumbnail().ok()?;
    let stream = thumbnail.OpenReadAsync().and_then(|f| f.get()).ok()?;

    let size = stream.Size().ok()?;
    if size == 0 || size > 10 * 1024 * 1024 {
        return None;
    }

    let buffer = Buffer::Create(size as u32).ok()?;
    let result_buffer = stream
        .ReadAsync(&buffer, size as u32, InputStreamOptions::ReadAhead)
        .and_then(|f| f.get())
        .ok()?;

    let reader = DataReader::FromBuffer(&result_buffer).ok()?;
    let length = result_buffer.Length().ok()? as usize;
    let mut data = vec![0u8; length];
    reader.ReadBytes(&mut data).ok()?;

    Some(data)
}
