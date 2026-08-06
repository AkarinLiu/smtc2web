use super::{generate_song_id, get_cached_album_art, matches_process_filter, set_cached_album_art};
use crate::{CURRENT_APP_DISPLAY_NAME, CURRENT_APP_ID, Shared, Song, format_duration};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use windows::Foundation::{EventRegistrationToken, TypedEventHandler};
use windows::Management::Deployment::PackageManager;
use windows::Media::Control::{
    CurrentSessionChangedEventArgs, GlobalSystemMediaTransportControlsSession as SmtcSession,
    GlobalSystemMediaTransportControlsSessionManager as SmtcManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus,
    MediaPropertiesChangedEventArgs, PlaybackInfoChangedEventArgs,
    TimelinePropertiesChangedEventArgs,
};
use windows::Storage::Streams::{Buffer, DataReader, InputStreamOptions};

static AUMID_DISPLAY_NAME_CACHE: std::sync::LazyLock<Mutex<HashMap<String, String>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

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

fn get_app_display_name(aumid: &str) -> String {
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
        get_store_app_display_name(aumid).unwrap_or_else(|| get_fallback_display_name(aumid))
    } else {
        get_fallback_display_name(aumid)
    };
    {
        let mut cache = AUMID_DISPLAY_NAME_CACHE.lock().unwrap();
        cache.insert(aumid.to_string(), display_name.clone());
    }
    display_name
}

fn fetch_thumbnail(session: &SmtcSession) -> Option<Vec<u8>> {
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

// ponytail: extracted from 3 copy-pasted blocks in update_full_state/playback/timeline
fn apply_timeline(song: &mut Song, session: &SmtcSession) {
    if let Ok(timeline) = session.GetTimelineProperties() {
        let pos = timeline.Position().unwrap().Duration;
        let dur = timeline.EndTime().unwrap().Duration;
        let pos_secs = (pos / 10_000_000) as u64;
        let dur_secs = (dur / 10_000_000) as u64;
        if dur_secs > 0 {
            song.position = Some(format_duration(pos_secs));
            song.duration = Some(format_duration(dur_secs));
            let pct = (pos_secs as f64 * 100.0) / dur_secs as f64;
            song.pct = Some((pct * 10.0).round() / 10.0);
        }
    }
}

// ponytail: stores only Send+Sync parts of current session; handlers leaked outside
struct CurrentSessionInfo {
    session: SmtcSession,
    playback_token: EventRegistrationToken,
    media_token: EventRegistrationToken,
    timeline_token: EventRegistrationToken,
}

struct EventContext {
    state: Shared,
    manager: SmtcManager,
    process_filter: String,
    current: Mutex<Option<CurrentSessionInfo>>,
    last_song: Mutex<Song>,
    last_position: Mutex<Option<String>>,
    last_song_id: Mutex<String>,
    last_art_update: Mutex<u64>,
}

impl EventContext {
    fn new(state: Shared, process_filter: &str) -> Result<Self, String> {
        let manager = SmtcManager::RequestAsync()
            .and_then(|f| f.get())
            .map_err(|e| format!("Failed to get SMTC session manager: {:?}", e))?;
        Ok(EventContext {
            state,
            manager,
            process_filter: process_filter.to_string(),
            current: Mutex::new(None),
            last_song: Mutex::new(Song::default()),
            last_position: Mutex::new(None),
            last_song_id: Mutex::new(String::new()),
            last_art_update: Mutex::new(0),
        })
    }

    fn on_session_changed(self: &Arc<Self>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Detach old session handlers
        {
            let mut current = self.current.lock().unwrap();
            if let Some(old) = current.take() {
                let _ = old.session.RemovePlaybackInfoChanged(old.playback_token);
                let _ = old.session.RemoveMediaPropertiesChanged(old.media_token);
                let _ = old
                    .session
                    .RemoveTimelinePropertiesChanged(old.timeline_token);
            }
        }

        // Get new session
        let session = match self.manager.GetCurrentSession() {
            Ok(s) => s,
            Err(_) => {
                self.clear_state();
                return;
            }
        };

        let app_id = session
            .SourceAppUserModelId()
            .ok()
            .map(|h| h.to_string())
            .unwrap_or_default();
        let app_name = get_app_display_name(&app_id);

        if !matches_process_filter(&self.process_filter, &app_id, &app_name) {
            self.clear_state();
            return;
        }

        *CURRENT_APP_ID.lock().unwrap() = app_id;
        *CURRENT_APP_DISPLAY_NAME.lock().unwrap() = app_name;

        self.update_full_state(&session, timestamp);

        // Subscribe to session events — handlers are leaked to stay alive
        let weak = Arc::downgrade(self);

        let playback_h: TypedEventHandler<SmtcSession, PlaybackInfoChangedEventArgs> = {
            let weak = weak.clone();
            TypedEventHandler::new(move |sender, _args| {
                if let (Some(ctx), Some(session)) = (weak.upgrade(), sender) {
                    let ts = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    ctx.update_playback(&session, ts);
                }
                Ok(())
            })
        };

        let media_h: TypedEventHandler<SmtcSession, MediaPropertiesChangedEventArgs> = {
            let weak = weak.clone();
            TypedEventHandler::new(move |sender, _args| {
                if let (Some(ctx), Some(session)) = (weak.upgrade(), sender) {
                    let ts = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    ctx.update_media(&session, ts);
                }
                Ok(())
            })
        };

        let timeline_h: TypedEventHandler<SmtcSession, TimelinePropertiesChangedEventArgs> = {
            TypedEventHandler::new(move |sender, _args| {
                if let (Some(ctx), Some(session)) = (weak.upgrade(), sender) {
                    ctx.update_timeline(&session);
                }
                Ok(())
            })
        };

        let playback_token = match session.PlaybackInfoChanged(&playback_h) {
            Ok(t) => t,
            Err(_) => return,
        };
        let media_token = match session.MediaPropertiesChanged(&media_h) {
            Ok(t) => t,
            Err(_) => {
                let _ = session.RemovePlaybackInfoChanged(playback_token);
                return;
            }
        };
        let timeline_token = match session.TimelinePropertiesChanged(&timeline_h) {
            Ok(t) => t,
            Err(_) => {
                let _ = session.RemovePlaybackInfoChanged(playback_token);
                let _ = session.RemoveMediaPropertiesChanged(media_token);
                return;
            }
        };

        // ponytail: leak handlers — WinRT keeps a COM ref too, but double-keep is safe
        let _ = Box::leak(Box::new(playback_h));
        let _ = Box::leak(Box::new(media_h));
        let _ = Box::leak(Box::new(timeline_h));

        *self.current.lock().unwrap() = Some(CurrentSessionInfo {
            session,
            playback_token,
            media_token,
            timeline_token,
        });
    }

    fn clear_state(&self) {
        *self.state.write().unwrap() = Song::default();
        *self.last_song.lock().unwrap() = Song::default();
        *self.last_position.lock().unwrap() = None;
        *CURRENT_APP_ID.lock().unwrap() = String::new();
        *CURRENT_APP_DISPLAY_NAME.lock().unwrap() = String::new();
    }

    // ponytail: extracted art caching logic shared by update_full_state and update_media
    fn update_song_art(&self, song: &mut Song, session: &SmtcSession, timestamp: u64) {
        let song_id = generate_song_id(&song.title, &song.artist, &song.album);
        let cached_art = get_cached_album_art(&song_id);
        let mut last_art_update = self.last_art_update.lock().unwrap();
        let last_song_id = self.last_song_id.lock().unwrap().clone();

        let should_fetch = cached_art.is_none()
            || (song_id != last_song_id && timestamp.saturating_sub(*last_art_update) > 30);

        if should_fetch {
            if let Some(data) = fetch_thumbnail(session) {
                use base64::{Engine, engine::general_purpose::STANDARD};
                let data_uri = format!("data:image/jpeg;base64,{}", STANDARD.encode(&data));
                set_cached_album_art(&song_id, data_uri.clone());
                song.album_art = Some(data_uri);
                *last_art_update = timestamp;
            }
        } else {
            song.album_art = cached_art;
        }

        if song_id != last_song_id {
            *self.last_song_id.lock().unwrap() = song_id;
        }
    }

    fn update_full_state(&self, session: &SmtcSession, timestamp: u64) {
        let mut song = Song {
            last_update: timestamp,
            ..Song::default()
        };

        let is_playing = session
            .GetPlaybackInfo()
            .ok()
            .and_then(|p| p.PlaybackStatus().ok())
            .map(|s| s == PlaybackStatus::Playing)
            .unwrap_or(false);
        song.is_playing = is_playing;

        if let Ok(media_info) = session.TryGetMediaPropertiesAsync().and_then(|f| f.get()) {
            song.title = media_info.Title().unwrap_or_default().to_string();
            song.artist = media_info.Artist().unwrap_or_default().to_string();
            song.album = media_info.AlbumTitle().unwrap_or_default().to_string();
        }

        apply_timeline(&mut song, session);

        self.update_song_art(&mut song, session, timestamp);

        *self.last_song.lock().unwrap() = song.clone();
        *self.last_position.lock().unwrap() = song.position.clone();
        *self.state.write().unwrap() = song;
    }

    fn update_playback(&self, session: &SmtcSession, timestamp: u64) {
        let is_playing = session
            .GetPlaybackInfo()
            .ok()
            .and_then(|p| p.PlaybackStatus().ok())
            .map(|s| s == PlaybackStatus::Playing)
            .unwrap_or(false);

        let mut song = self.last_song.lock().unwrap().clone();
        song.is_playing = is_playing;
        song.last_update = timestamp;

        apply_timeline(&mut song, session);

        *self.last_song.lock().unwrap() = song.clone();
        *self.last_position.lock().unwrap() = song.position.clone();
        *self.state.write().unwrap() = song;
    }

    fn update_media(&self, session: &SmtcSession, timestamp: u64) {
        let mut song = self.last_song.lock().unwrap().clone();
        song.last_update = timestamp;

        if let Ok(media_info) = session.TryGetMediaPropertiesAsync().and_then(|f| f.get()) {
            song.title = media_info.Title().unwrap_or_default().to_string();
            song.artist = media_info.Artist().unwrap_or_default().to_string();
            song.album = media_info.AlbumTitle().unwrap_or_default().to_string();
        }

        self.update_song_art(&mut song, session, timestamp);

        *self.last_song.lock().unwrap() = song.clone();
        *self.state.write().unwrap() = song;
    }

    fn update_timeline(&self, session: &SmtcSession) {
        let mut song = self.last_song.lock().unwrap().clone();
        apply_timeline(&mut song, session);
        *self.last_song.lock().unwrap() = song.clone();
        *self.last_position.lock().unwrap() = song.position.clone();
        *self.state.write().unwrap() = song;
    }
}

pub fn run_event_driven(state: Shared, process_filter: &str) -> Result<(), String> {
    let ctx = Arc::new(EventContext::new(state, process_filter)?);

    // Subscribe to session change — handler leaked to stay alive
    let weak = Arc::downgrade(&ctx);
    let handler: TypedEventHandler<SmtcManager, CurrentSessionChangedEventArgs> =
        TypedEventHandler::new(move |_sender, _args| {
            if let Some(ctx) = weak.upgrade() {
                ctx.on_session_changed();
            }
            Ok(())
        });

    let _token = ctx
        .manager
        .CurrentSessionChanged(&handler)
        .map_err(|e| format!("Failed to subscribe CurrentSessionChanged: {:?}", e))?;

    // ponytail: leak handler — WinRT also holds a COM ref, double-keep is safe
    let _ = Box::leak(Box::new(handler));

    ctx.on_session_changed();

    loop {
        std::thread::park();
    }
}
