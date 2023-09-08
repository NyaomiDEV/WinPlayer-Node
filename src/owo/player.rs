use std::time::Duration;

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

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

#[allow(clippy::enum_variant_names)]
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

#[allow(dead_code)] // per ora??
pub struct Player {
    session: GlobalSystemMediaTransportControlsSession,
    aumid: String,

    tx: UnboundedSender<PlayerEvent>,
    rx: UnboundedReceiver<PlayerEvent>,

    event_tokens: EventToken,
}

impl Player {
    pub fn new(session: GlobalSystemMediaTransportControlsSession, aumid: String) -> Self {
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

        let playback_info_changed_token = session
            .PlaybackInfoChanged(&playback_info_changed_handler)
            .unwrap_or_default();
        let media_properties_changed_token = session
            .MediaPropertiesChanged(&media_properties_changed_handler)
            .unwrap_or_default();
        let timeline_properties_changed_token = session
            .TimelinePropertiesChanged(&timeline_properties_changed_handler)
            .unwrap_or_default();

        let event_tokens = EventToken {
            playback_info_changed_token,
            media_properties_changed_token,
            timeline_properties_changed_token,
        };

        Player {
            session,
            aumid,

            tx,
            rx,

            event_tokens,
        }
    }

    pub async fn poll_next_event(&mut self) -> Option<PlayerEvent> {
        self.rx.recv().await
    }

    pub async fn get_status(&self) -> Status {
        let playback_info = self.session.GetPlaybackInfo();
        let timeline_properties = self.session.GetTimelineProperties();

        Status {
            metadata: get_session_metadata(&self.session),
            capabilities: get_session_capabilities(&self.session),
            status: 'rt: {
                if let Ok(playback_info) = playback_info.as_ref() {
                    if let Ok(status) = playback_info.PlaybackStatus() {
                        match status {
                            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => {
                                break 'rt String::from("Playing")
                            }
                            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => {
                                break 'rt String::from("Paused")
                            }
                            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => {
                                break 'rt String::from("Stopped")
                            }
                            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Changing => {
                                break 'rt String::from("Changing")
                            }
                            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed => {
                                break 'rt String::from("Closed")
                            }
                            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Opened => {
                                break 'rt String::from("Opened")
                            }
                            _ => break 'rt String::from("Unknown"),
                        }
                    }
                }
                String::from("Unknown")
            },
            is_loop: 'rt: {
                if let Ok(playback_info) = playback_info.as_ref() {
                    if let Ok(_mode) = playback_info.AutoRepeatMode() {
                        if let Ok(value) = _mode.Value() {
                            match value {
                                MediaPlaybackAutoRepeatMode::List => {
                                    break 'rt String::from("List")
                                }
                                MediaPlaybackAutoRepeatMode::Track => {
                                    break 'rt String::from("Track")
                                }
                                _ => break 'rt String::from("None"),
                            }
                        }
                    }
                }
                String::from("None")
            },
            shuffle: 'rt: {
                if let Ok(playback_info) = playback_info.as_ref() {
                    if let Ok(shuffle) = playback_info.IsShuffleActive() {
                        break 'rt shuffle.Value().unwrap_or(false);
                    }
                }
                false
            },
            volume: -1f64,
            elapsed: compute_position(
                timeline_properties.ok().as_ref(),
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

    pub fn get_playback_status(&self) -> GlobalSystemMediaTransportControlsSessionPlaybackStatus {
        if let Ok(playback_info) = self.session.GetPlaybackInfo() {
            if let Ok(playback_status) = playback_info.PlaybackStatus() {
                return playback_status;
            }
        }
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped
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

    pub async fn seek(&self, offset_s: f64) -> bool {
        if let Ok(timeline_properties) = self.session.GetTimelineProperties() {
            let position = 'rt: {
                if let Ok(_pos) = timeline_properties.Position() {
                    let _duration: Duration = _pos.into();
                    break 'rt _duration.as_secs_f64();
                }
                0f64
            };
            return self.set_position(position + offset_s).await;
        }

        false
    }

    pub async fn seek_percentage(&self, percentage: f64) -> bool {
        if let Ok(timeline_properties) = self.session.GetTimelineProperties() {
            let start_time = 'rt: {
                if let Ok(_start) = timeline_properties.StartTime() {
                    let _duration: Duration = _start.into();
                    break 'rt _duration.as_secs_f64();
                }
                0f64
            };

            let end_time = 'rt: {
                if let Ok(_end) = timeline_properties.EndTime() {
                    let _duration: Duration = _end.into();
                    break 'rt _duration.as_secs_f64();
                }
                0f64
            };

            let length = end_time - start_time;
            return self.set_position(length * percentage).await;
        }
        false
    }

    pub async fn set_position(&self, position_s: f64) -> bool {
        if let Ok(result) = self
            .session
            .TryChangePlaybackPositionAsync((position_s * 1e+7f64) as i64)
        {
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
        let _ = self
            .session
            .RemoveMediaPropertiesChanged(self.event_tokens.media_properties_changed_token);
        let _ = self
            .session
            .RemovePlaybackInfoChanged(self.event_tokens.playback_info_changed_token);
        let _ = self
            .session
            .RemoveTimelinePropertiesChanged(self.event_tokens.timeline_properties_changed_token);
    }
}
