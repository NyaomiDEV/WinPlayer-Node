use std::{
    rc::{Rc, Weak},
    sync::{Arc, Mutex},
};

use chrono;

use windows::{
    ApplicationModel, Foundation,
    Foundation::{Collections, TypedEventHandler},
    Graphics::Imaging,
    Media::Control::{
        self, GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus, SessionsChangedEventArgs,
    },
    Media::MediaPlaybackAutoRepeatMode,
    Security::Cryptography::Core,
    Storage::Streams,
    System,
};

struct Player {
    session_manager: Control::GlobalSystemMediaTransportControlsSessionManager,
    active_player: Option<String>,
}

impl Player {
    pub async fn new() -> Self {
        let session_manager =
            Control::GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                .expect("The session manager is kil")
                .await
                .expect("The session manager is kil 2");

        Player {
            session_manager,
            active_player: None,
        }
    }

    pub fn run(mut self) {
        let rc_self = Arc::new(Mutex::new(self));

        let handler = TypedEventHandler::new({
            let s = rc_self.clone();
            move |_, _| {
                Ok({
                    let mut binding = s.lock();
                    let s = binding.as_mut().unwrap();
                    let preferred = s.active_player.clone();
                    s.update_active_player(preferred);
                })
            }
        });

        rc_self
            .lock()
            .unwrap()
            .session_manager
            .SessionsChanged(&handler);

        let preferred = rc_self.lock().unwrap().active_player.clone();
        rc_self.lock().unwrap().update_active_player(preferred);
    }

    async fn get_player_name(
        player: Control::GlobalSystemMediaTransportControlsSession,
    ) -> Option<String> {
        let mut player_name = player.SourceAppUserModelId().unwrap();
        // TODO: Match all this madness and just return None if Err
        let user = System::User::FindAllAsync()
            .expect("AO ergo Viola e Argo sono cute")
            .await
            .expect("FANCULO")
            .GetAt(0)
            .expect("NON POSSO PRENDERE L'UTENTE");

        // TODO: Match all this madness and just return None if Err
        player_name = ApplicationModel::AppInfo::GetFromAppUserModelIdForUser(&user, &player_name)
            .expect("ERR")
            .DisplayInfo()
            .expect("CHE PALLE")
            .DisplayName()
            .expect("DIO CANE");

        if player.SourceAppUserModelId().unwrap() == player_name
            && player_name.to_string().ends_with(".exe")
        {
            let without_exe_at_end =
                String::from(player_name.to_string().strip_suffix(".exe").unwrap());
            return Some(without_exe_at_end);
        }

        Some(player_name.to_string()) // ok come torniamo none a ogni expect?
    }

    fn get_player_session(&self) -> Option<Control::GlobalSystemMediaTransportControlsSession> {
        match self.session_manager.GetSessions() {
            Ok(ses) => {
                let active_player = self.active_player.clone().unwrap_or_default();
                for session in ses {
                    if session.SourceAppUserModelId().unwrap().to_string() == active_player {
                        return Some(session);
                    }
                }
            }
            Err(e) => return None,
        }
        None
    }

    fn update_active_player(&mut self, preferred: Option<String>) {
        self.active_player = None;

        let sessions = self.session_manager.GetSessions().expect("No sessions?");
        let preferred = preferred.unwrap_or_default();
        for session in sessions {
            if (session.SourceAppUserModelId().unwrap().to_string() == preferred)
                || session.GetPlaybackInfo().unwrap().PlaybackStatus().unwrap()
                    == Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
            {
                self.active_player = Some(session.SourceAppUserModelId().unwrap().to_string());
                break;
            }
        }
    }

    async fn play(&self) -> Result<bool, &str> {
        match self.get_player_session() {
            None => Err("No player"),
            Some(session) => match session.TryPlayAsync() {
                Ok(result) => return Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    async fn pause(&self) -> Result<bool, &str> {
        match self.get_player_session() {
            None => Err("No player"),
            Some(session) => match session.TryPauseAsync() {
                Ok(result) => return Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    async fn play_pause(&self) -> Result<bool, &str> {
        match self.get_player_session() {
            None => Err("No player"),
            Some(session) => match session.TryTogglePlayPauseAsync() {
                Ok(result) => return Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    async fn stop(&self) -> Result<bool, &str> {
        match self.get_player_session() {
            None => Err("No player"),
            Some(session) => match session.TryStopAsync() {
                Ok(result) => return Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    async fn next(&self) -> Result<bool, &str> {
        match self.get_player_session() {
            None => Err("No player"),
            Some(session) => match session.TrySkipNextAsync() {
                Ok(result) => return Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    async fn previous(&self) -> Result<bool, &str> {
        match self.get_player_session() {
            None => Err("No player"),
            Some(session) => match session.TrySkipPreviousAsync() {
                Ok(result) => return Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    async fn shuffle(&self) -> Result<bool, &str> {
        match self.get_player_session() {
            None => Err("No player"),
            Some(session) => match session.GetPlaybackInfo() {
                Ok(playback_info) => match playback_info.IsShuffleActive() {
                    Ok(shuffle_active) => {
                        match session
                            .TryChangeShuffleActiveAsync(shuffle_active.Value().unwrap_or(false))
                        {
                            Ok(async_operation) => match async_operation.await {
                                Ok(result) => Ok(result),
                                Err(_) => Ok(false),
                            },
                            Err(_) => Err("Could not change shuffle mode"),
                        }
                    }
                    Err(_) => Ok(false),
                },
                Err(_) => Err("Could not get playback info"),
            },
        }
    }

    async fn repeat(&self) -> Result<bool, &str> {
        match self.get_player_session() {
            None => Err("No player"),
            Some(session) => match session.GetPlaybackInfo() {
                Ok(playback_info) => match playback_info.AutoRepeatMode() {
                    Ok(repeat_mode) => {
                        let new_repeat_mode = match repeat_mode.Value() {
                            Err(_) => MediaPlaybackAutoRepeatMode::None,
                            Ok(rp) => match rp {
                                MediaPlaybackAutoRepeatMode::None => {
                                    MediaPlaybackAutoRepeatMode::List
                                }
                                MediaPlaybackAutoRepeatMode::List => {
                                    MediaPlaybackAutoRepeatMode::Track
                                }
                                MediaPlaybackAutoRepeatMode::Track => {
                                    MediaPlaybackAutoRepeatMode::None
                                }
                                _ => MediaPlaybackAutoRepeatMode::None,
                            },
                        };
                        match session.TryChangeAutoRepeatModeAsync(new_repeat_mode) {
                            Ok(async_operation) => match async_operation.await {
                                Ok(result) => Ok(result),
                                Err(_) => Ok(false),
                            },
                            Err(_) => Err("Could not change repeat mode"),
                        }
                    }
                    Err(_) => Ok(false),
                },
                Err(_) => Err("Could not get playback info"),
            },
        }
    }

    async fn seek(&self, offset_us: isize) -> Result<bool, &str> {
        match self.get_player_session() {
            None => Err("No player"),
            Some(session) => match session.GetTimelineProperties() {
                Ok(timeline_properties) => {
                    let position = timeline_properties.Position().unwrap_or_default();
                    // no no fai al contrario
                    // vuole un f32, a okay lol
                    // vabbe
                    // non sta la doc di windows ?
                    // A che sarebbero /a dbejkabakjhdjg kjhsgfshjfgakjfgkjhsdgfkhsdjgfkjashfgksjhgfkjhsgd
                    // si cosi pusho e vado a nanna
                    return self.set_position(position.Duration + offset_us).await;
                }
                Err(_) => Err("Could not get timeline properties"),
            },
        }
    }

    async fn seek_percentage(&self, percentage: f32) -> Result<bool, &str> {
        match self.get_player_session() {
            None => Err("No player"),
            Some(session) => match session.GetTimelineProperties() {
                Ok(timeline_properties) => {
                    let start_time = timeline_properties.StartTime().unwrap_or_default();
                    let end_time = timeline_properties.EndTime().unwrap_or_default();
                    let length = (end_time.Duration - start_time.Duration) as f32 / 1000.0;
                    return self.set_position(length * percentage).await;
                }
                Err(_) => Err("Could not get timeline properties"),
            },
        }
    }

    async fn set_position(&self, position_s: f32) -> Result<bool, &str> {
        match self.get_player_session() {
            None => Err("No player"),
            Some(session) => {
                match session.TryChangePlaybackPositionAsync((position_s * 1000f32) as i64) {
                    // probabilmente non worka e la pos sara' wonky
                    Ok(result) => return Ok(result.await.unwrap_or(false)),
                    Err(_) => Err("Error while trying to perform the command"),
                }
            }
        }
    }

    async fn get_position(&self) -> Result<f32, &str> {
        match self.get_player_session() {
            None => Err("No player"),
            Some(session) => match session.GetTimelineProperties() {
                Ok(timeline_properties) => {
                    if timeline_properties.EndTime().unwrap_or_default().Duration == 0 {
                        return Ok(0f32);
                    }

                    let mut position = timeline_properties.Position().unwrap_or_default().Duration;
                    let playback_status =
                        session.GetPlaybackInfo().unwrap().PlaybackStatus().unwrap();

                    if playback_status
                        == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
                    {
                        let time_from_last_update = chrono::offset::Utc::now()
                            - timeline_properties
                                .LastUpdatedTime()
                                .unwrap_or(Foundation::DateTime { UniversalTime: 0 })
                                .UniversalTime;
                        position += time_from_last_update;
                    }

                    Ok(
                        (position as f32
                            - timeline_properties.StartTime().unwrap().Duration as f32)
                            / 1000f32,
                    )
                }
                Err(_) => Err("Could not get timeline properties"),
            },
        }
    }
}
