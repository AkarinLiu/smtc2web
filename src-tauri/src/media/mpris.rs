use super::{generate_song_id, get_cached_album_art, matches_process_filter, set_cached_album_art, MediaSession, SessionInfo};
use mpris::PlaybackStatus;
use std::sync::Mutex;

pub struct MprisSession {
    process_filter: String,
    last_art_url: Mutex<Option<String>>,
}

impl MprisSession {
    fn find_player(&self) -> Result<mpris::Player, String> {
        let finder = mpris::PlayerFinder::new()
            .map_err(|e| format!("Failed to create PlayerFinder: {:?}", e))?;

        let players: Vec<_> = finder
            .iter_players()
            .map_err(|e| format!("Failed to iterate players: {:?}", e))?
            .filter_map(|p| p.ok())
            .collect();

        crate::log_info!("MPRIS2: {} players found", players.len());

        if players.is_empty() {
            return Err("No players found".to_string());
        }

        let mut sorted_players = players;
        sorted_players.sort_by(|a, b| {
            let a_status = a.get_playback_status().unwrap_or(PlaybackStatus::Stopped);
            let b_status = b.get_playback_status().unwrap_or(PlaybackStatus::Stopped);
            
            match (a_status, b_status) {
                (PlaybackStatus::Playing, PlaybackStatus::Playing) => std::cmp::Ordering::Equal,
                (PlaybackStatus::Playing, _) => std::cmp::Ordering::Less,
                (_, PlaybackStatus::Playing) => std::cmp::Ordering::Greater,
                (PlaybackStatus::Paused, PlaybackStatus::Paused) => std::cmp::Ordering::Equal,
                (PlaybackStatus::Paused, _) => std::cmp::Ordering::Less,
                (_, PlaybackStatus::Paused) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        });

        for player in sorted_players {
            if matches_process_filter(&self.process_filter, &player.bus_name(), &player.identity()) {
                crate::log_debug!("MPRIS2: Selected player: {}", player.bus_name());
                return Ok(player);
            }
        }

        Err("No matching player found".to_string())
    }

    fn format_artist_list(artists: &[&str]) -> String {
        artists.join(", ")
    }

    fn extract_display_name(player: &mpris::Player) -> String {
        player.identity().to_string()
    }
}

impl MediaSession for MprisSession {
    fn new(process_filter: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        Ok(MprisSession {
            process_filter: process_filter.to_string(),
            last_art_url: Mutex::new(None),
        })
    }

    fn poll_current(&self) -> Option<SessionInfo> {
        let player = match self.find_player() {
            Ok(p) => p,
            Err(e) => {
                crate::log_debug!("MPRIS2: {}", e);
                return None;
            }
        };

        let metadata = match player.get_metadata() {
            Ok(m) => m,
            Err(e) => {
                crate::log_debug!("MPRIS2: Failed to get metadata: {:?}", e);
                return None;
            }
        };

        let title = metadata.title().unwrap_or_default().to_string();
        let artists = metadata.artists().unwrap_or_default();
        let artist = Self::format_artist_list(&artists);
        let album = metadata.album_name().unwrap_or_default().to_string();
        let art_url = metadata.art_url().map(|u| u.to_string()).unwrap_or_default();
        let length_us = metadata.length().map(|d| d.as_micros() as i64).unwrap_or(0);

        let status = player.get_playback_status().unwrap_or(PlaybackStatus::Stopped);
        let is_playing = status == PlaybackStatus::Playing;

        let position_us = player.get_position()
            .map(|d| d.as_micros() as i64)
            .unwrap_or(0);

        let display_name = Self::extract_display_name(&player);
        let app_id = player.bus_name().to_string();

        if let Ok(mut cached_url) = self.last_art_url.lock() {
            if !art_url.is_empty() {
                *cached_url = Some(art_url.clone());
            }
        }

        crate::log_info!(
            "MPRIS2: {} - {} by {} [{}]",
            display_name,
            title,
            artist,
            if is_playing { "playing" } else { "paused" }
        );

        Some(SessionInfo {
            title,
            artist,
            album,
            is_playing,
            position_secs: (position_us / 1_000_000).max(0) as u64,
            duration_secs: (length_us / 1_000_000).max(0) as u64,
            app_id,
            app_name: display_name,
        })
    }

    fn get_album_art_base64(&self, artist: &str, title: &str, album: &str) -> Option<String> {
        let song_id = generate_song_id(title, artist, album);

        if let Some(cached) = get_cached_album_art(&song_id) {
            return Some(cached);
        }

        let art_url = self.last_art_url.lock().ok()?.clone()?;

        let file_path = art_url.strip_prefix("file://")?;

        let data = std::fs::read(file_path).ok()?;

        use base64::{engine::general_purpose::STANDARD, Engine};
        let mime = mime_guess::from_path(file_path)
            .first_or_octet_stream()
            .to_string();
        let data_uri = format!("data:{};base64,{}", mime, STANDARD.encode(&data));

        set_cached_album_art(&song_id, data_uri.clone());

        Some(data_uri)
    }
}
