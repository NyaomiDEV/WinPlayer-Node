use windows::{
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Media::MediaPlaybackAutoRepeatMode,
};

use crate::types::{Position, Update};

use crate::util::{
    compute_position, get_session_capabilities, get_session_metadata, get_session_player_name,
};

pub struct Player {
    pub session: GlobalSystemMediaTransportControlsSession,
}

impl Player {
    pub fn new(session: GlobalSystemMediaTransportControlsSession) -> Self {
        Player { session }
    }

    pub async fn get_session_status(&self) -> Update {
        let playback_info = self.session.GetPlaybackInfo().ok();
        let timeline_properties = self.session.GetTimelineProperties().ok();

        Update {
            metadata: get_session_metadata(&self.session).await,
            capabilities: get_session_capabilities(&self.session),
            status: 'rt: {
                if playback_info.is_none() {
                    break 'rt String::from("Stopped");
                }
                let _status = playback_info.as_ref().unwrap().PlaybackStatus().ok();
                match _status.unwrap() {
                    GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => {
                        String::from("Playing")
                    }
                    GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => {
                        String::from("Paused")
                    }
                    _ => String::from("Stopped"),
                }
            },
            is_loop: 'rt: {
                if playback_info.is_none() {
                    break 'rt String::from("None");
                }
                let _mode = playback_info.as_ref().unwrap().AutoRepeatMode().ok();
                if _mode.is_none() {
                    break 'rt String::from("None");
                }
                match _mode
                    .unwrap()
                    .Value()
                    .unwrap_or(MediaPlaybackAutoRepeatMode::None)
                {
                    MediaPlaybackAutoRepeatMode::List => String::from("List"),
                    MediaPlaybackAutoRepeatMode::Track => String::from("Track"),
                    _ => String::from("None"),
                }
            },
            shuffle: 'rt: {
                if playback_info.is_none() {
                    break 'rt false;
                }
                let _shuffle = playback_info.as_ref().unwrap().IsShuffleActive().ok();
                if _shuffle.is_none() {
                    break 'rt false;
                }
                _shuffle.unwrap().Value().unwrap_or(false)
            },
            volume: -1f64,
            elapsed: compute_position(timeline_properties.as_ref(), playback_info.as_ref(), false),
            app: 'rt: {
                let aumid = self.session.SourceAppUserModelId().ok();
                if aumid.is_none() {
                    break 'rt None::<String>;
                }
                Some(aumid.unwrap().to_string())
            },
            app_name: 'rt: {
                let app_name = get_session_player_name(&self.session).await.ok();
                if app_name.is_none() {
                    break 'rt None::<String>;
                }
                Some(app_name.unwrap())
            },
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

    pub async fn get_position(&self) -> Option<Position> {
        return compute_position(
            self.session.GetTimelineProperties().ok().as_ref(),
            self.session.GetPlaybackInfo().ok().as_ref(),
            true,
        );
    }
}
