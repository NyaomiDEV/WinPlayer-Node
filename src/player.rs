use windows::{
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Media::MediaPlaybackAutoRepeatMode,
};

use crate::types::{Position, Status};

use crate::util::{
    compute_position, get_session_capabilities, get_session_metadata, get_session_player_name,
};

pub struct Player {
    pub session: GlobalSystemMediaTransportControlsSession,
    pub aumid: String,
    pub friendly_name: Option<String>,
}

impl Player {
    pub async fn new(session: GlobalSystemMediaTransportControlsSession, aumid: String) -> Self {
        Player {
            friendly_name: get_session_player_name(&aumid).await,
            session,
            aumid,
        }
    }

    pub async fn get_session_status(&self) -> Status {
        let playback_info = self.session.GetPlaybackInfo();
        let timeline_properties = self.session.GetTimelineProperties().ok();

        Status {
            metadata: get_session_metadata(&self.session).await,
            capabilities: get_session_capabilities(&self.session),
            status: 'rt: {
                if playback_info.is_err() {
                    break 'rt String::from("Stopped");
                }
                let status = playback_info.as_ref().unwrap().PlaybackStatus();
                match status {
                    Ok(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing) => {
                        String::from("Playing")
                    }
                    Ok(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused) => {
                        String::from("Paused")
                    }
                    _ => String::from("Stopped"),
                }
            },
            is_loop: 'rt: {
                if playback_info.is_err() {
                    break 'rt String::from("None");
                }
                let _mode = playback_info.as_ref().unwrap().AutoRepeatMode();
                if _mode.is_err() {
                    break 'rt String::from("None");
                }
                match _mode.unwrap().Value() {
                    Ok(MediaPlaybackAutoRepeatMode::List) => String::from("List"),
                    Ok(MediaPlaybackAutoRepeatMode::Track) => String::from("Track"),
                    _ => String::from("None"),
                }
            },
            shuffle: 'rt: {
                if playback_info.is_err() {
                    break 'rt false;
                }
                let _shuffle = playback_info.as_ref().unwrap().IsShuffleActive().ok();
                if _shuffle.is_none() {
                    break 'rt false;
                }
                _shuffle.unwrap().Value().unwrap_or(false)
            },
            volume: -1f64,
            elapsed: compute_position(
                timeline_properties.as_ref(),
                playback_info.ok().as_ref(),
                false,
            ),
            app: Some(self.aumid.clone()),
            app_name: self.friendly_name.clone(),
        }
    }

    pub async fn play(&self) -> bool {
        if let Ok(result) = self.session.TryPlayAsync() {
            return result.await.unwrap_or(false);
        }
        false
    }

    pub async fn pause(&self) -> bool {
        if let Ok(result) = self.session.TryPauseAsync() {
            return result.await.unwrap_or(false);
        }

        false
    }

    pub async fn play_pause(&self) -> bool {
        if let Ok(result) = self.session.TryTogglePlayPauseAsync() {
            return result.await.unwrap_or(false);
        }
        false
    }

    pub async fn stop(&self) -> bool {
        if let Ok(result) = self.session.TryStopAsync() {
            return result.await.unwrap_or(false);
        }
        false
    }

    pub async fn next(&self) -> bool {
        if let Ok(result) = self.session.TrySkipNextAsync() {
            return result.await.unwrap_or(false);
        }
        false
    }

    pub async fn previous(&self) -> bool {
        if let Ok(result) = self.session.TrySkipPreviousAsync() {
            return result.await.unwrap_or(false);
        }
        false
    }

    pub async fn set_shuffle(&self, value: bool) -> bool {
        if let Ok(result) = self.session.TryChangeShuffleActiveAsync(value) {
            return result.await.unwrap_or(false);
        }
        false
    }

    pub async fn shuffle(&self) -> bool {
        if let Ok(playback_info) = self.session.GetPlaybackInfo() {
            if let Ok(shuffle_active) = playback_info.IsShuffleActive() {
                if let Ok(result) = self
                    .session
                    .TryChangeShuffleActiveAsync(shuffle_active.Value().unwrap_or(false))
                {
                    return result.await.unwrap_or(false);
                }
            }
        }
        false
    }

    pub async fn set_repeat(&self, value: MediaPlaybackAutoRepeatMode) -> bool {
        if let Ok(result) = self.session.TryChangeAutoRepeatModeAsync(value) {
            return result.await.unwrap_or(false);
        }
        false
    }

    pub async fn repeat(&self) -> bool {
        if let Ok(playback_info) = self.session.GetPlaybackInfo() {
            if let Ok(repeat_mode) = playback_info.AutoRepeatMode() {
                let new_repeat_mode = match repeat_mode.Value() {
                    Err(_) => MediaPlaybackAutoRepeatMode::None,
                    Ok(rp) => match rp {
                        MediaPlaybackAutoRepeatMode::None => MediaPlaybackAutoRepeatMode::List,
                        MediaPlaybackAutoRepeatMode::List => MediaPlaybackAutoRepeatMode::Track,
                        MediaPlaybackAutoRepeatMode::Track => MediaPlaybackAutoRepeatMode::None,
                        _ => MediaPlaybackAutoRepeatMode::None,
                    },
                };
                if let Ok(result) = self.session.TryChangeAutoRepeatModeAsync(new_repeat_mode) {
                    return result.await.unwrap_or(false);
                }
            }
        }
        false
    }

    pub async fn seek(&self, offset_us: i64) -> bool {
        if let Ok(timeline_properties) = self.session.GetTimelineProperties() {
            if let Ok(position) = timeline_properties.Position() {
                return self
                    .set_position((position.Duration + offset_us) as f64 / 1000f64)
                    .await;
            }
        }

        false
    }

    pub async fn seek_percentage(&self, percentage: f64) -> bool {
        if let Ok(timeline_properties) = self.session.GetTimelineProperties() {
            let start_time = timeline_properties.StartTime().unwrap_or_default();
            let end_time = timeline_properties.EndTime().unwrap_or_default();
            let length = (end_time.Duration - start_time.Duration) as f64 / 1000.0;
            return self.set_position(length * percentage).await;
        }
        false
    }

    pub async fn set_position(&self, position_s: f64) -> bool {
        if let Ok(result) = self
            .session
            .TryChangePlaybackPositionAsync((position_s * 1000.0) as i64)
        {
            // probabilmente non worka e la pos sara' wonky
            return result.await.unwrap_or(false);
        }
        false
    }

    pub async fn get_position(&self, wants_current_position: bool) -> Option<Position> {
        return compute_position(
            self.session.GetTimelineProperties().ok().as_ref(),
            self.session.GetPlaybackInfo().ok().as_ref(),
            wants_current_position,
        );
    }
}
