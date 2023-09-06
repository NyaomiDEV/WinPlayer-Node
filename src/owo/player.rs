use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use windows::{
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Media::MediaPlaybackAutoRepeatMode,
};

use crate::owo::types::{Position, Status};

use crate::owo::util::{compute_position, get_session_capabilities, get_session_metadata};

pub enum PlayerEvent {
    PlaybackInfoChanged,
    MediaPropertiesChanged,
    TimelinePropertiesChanged,
}

struct EventToken {
    playback_info_changed_token: EventRegistrationToken,
    media_properties_changed_token: EventRegistrationToken,
    timeline_properties_changed_token: EventRegistrationToken,
}

pub struct Player {
    session: GlobalSystemMediaTransportControlsSession,
    aumid: String,

    event_tokens: Option<EventToken>,
}

impl Player {
    pub fn new(session: GlobalSystemMediaTransportControlsSession, aumid: String) -> Self {
        Player {
            session: session.clone(),
            aumid,

            event_tokens: None,
        }
    }

    pub fn set_events(&mut self) -> UnboundedReceiver<PlayerEvent> {
        if let Some(_tokens) = &self.event_tokens {
            // deregister to register again, invalidates stuff
            self.unset_events();
        }

        let (tx, rx) = unbounded_channel();

        let playback_info_changed_handler = TypedEventHandler::new({
            let tx = tx.clone();
            move |_, _| {
                let _ = tx.send(PlayerEvent::PlaybackInfoChanged);
                Ok(())
            }
        });

        let media_properties_changed_handler = TypedEventHandler::new({
            let tx = tx.clone();
            move |_, _| {
                let _ = tx.send(PlayerEvent::MediaPropertiesChanged);
                Ok(())
            }
        });

        let timeline_properties_changed_handler = TypedEventHandler::new({
            let tx = tx.clone();
            move |_, _| {
                let _ = tx.send(PlayerEvent::TimelinePropertiesChanged);
                Ok(())
            }
        });

        let playback_info_changed_token = self
            .session
            .PlaybackInfoChanged(&playback_info_changed_handler)
            .unwrap();
        let media_properties_changed_token = self
            .session
            .MediaPropertiesChanged(&media_properties_changed_handler)
            .unwrap();
        let timeline_properties_changed_token = self
            .session
            .TimelinePropertiesChanged(&timeline_properties_changed_handler)
            .unwrap();

        self.event_tokens = Some(EventToken {
            playback_info_changed_token,
            media_properties_changed_token,
            timeline_properties_changed_token,
        });

        rx
    }

    pub fn unset_events(&mut self) {
        let _ = self.session.RemoveMediaPropertiesChanged(
            self.event_tokens
                .as_mut()
                .unwrap()
                .media_properties_changed_token,
        );
        let _ = self.session.RemovePlaybackInfoChanged(
            self.event_tokens
                .as_mut()
                .unwrap()
                .playback_info_changed_token,
        );
        let _ = self.session.RemoveTimelinePropertiesChanged(
            self.event_tokens
                .as_mut()
                .unwrap()
                .timeline_properties_changed_token,
        );

        self.event_tokens = None;
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
        }
    }

    pub fn get_aumid(&self) -> String {
        self.aumid.clone()
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

    pub fn get_shuffle(&self) -> bool {
        if let Ok(playback_info) = self.session.GetPlaybackInfo() {
            if let Ok(shuffle_active) = playback_info.IsShuffleActive() {
                return shuffle_active.Value().unwrap_or(false);
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

    pub fn get_repeat(&self) -> Option<MediaPlaybackAutoRepeatMode> {
        if let Ok(playback_info) = self.session.GetPlaybackInfo() {
            if let Ok(repeat_mode) = playback_info.AutoRepeatMode() {
                return repeat_mode.Value().ok();
            }
        }
        None
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

impl Drop for Player {
    fn drop(&mut self) {
        self.unset_events();
    }
}
