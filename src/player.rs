use std::sync::{Arc, Mutex};

use chrono;

use windows::{
    ApplicationModel, Foundation,
    Foundation::{Collections, TypedEventHandler},
    Graphics::Imaging,
    Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Media::MediaPlaybackAutoRepeatMode,
    Security::Cryptography::Core,
    Storage::Streams,
    System,
};

use crate::types::Capabilities;

struct Player {
    session_manager: GlobalSystemMediaTransportControlsSessionManager,
    active_player: Option<String>,
}

impl Player {
    pub async fn new() -> Self {
        let session_manager =
            GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                .expect("The session manager is kil")
                .await
                .expect("The session manager is kil 2");

        Player {
            session_manager,
            active_player: None,
        }
    }

    pub fn run(self) { // Possiamo autostartarla dal costruttore o integrarla a esso?
        // Passando self così non rischiamo di perdercelo dopo questa call?
        let rc_self = Arc::new(Mutex::new(self));

        let handler = TypedEventHandler::new({
            let s = rc_self.clone();
            move |_, _| {
                Ok({
                    let mut binding = s.lock();
                    let s = binding.as_mut().unwrap();
                    let preferred = s.active_player.clone();
                    s.update_active_session(preferred);
                })
            }
        });

        rc_self
            .lock()
            .unwrap()
            .session_manager
            .SessionsChanged(&handler);

        let preferred = rc_self.lock().unwrap().active_player.clone();
        rc_self.lock().unwrap().update_active_session(preferred);
    }

    async fn get_session_player_name(
        session: GlobalSystemMediaTransportControlsSession,
    ) -> Option<String> {
        let mut player_name = session.SourceAppUserModelId().unwrap();
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

        if session.SourceAppUserModelId().unwrap() == player_name
            && player_name.to_string().ends_with(".exe")
        {
            let without_exe_at_end =
                String::from(player_name.to_string().strip_suffix(".exe").unwrap());
            return Some(without_exe_at_end);
        }

        Some(player_name.to_string()) // ok come torniamo none a ogni expect?
    }

    fn get_session(&self) -> Option<GlobalSystemMediaTransportControlsSession> {
        match self.session_manager.GetSessions() {
            Ok(ses) => {
                let active_player = self.active_player.clone().unwrap_or_default();
                for session in ses {
                    if session.SourceAppUserModelId().unwrap().to_string() == active_player {
                        return Some(session)
                    }
                }
                None
            }
            Err(e) => None,
        }
    }

    fn update_active_session(&mut self, preferred: Option<String>) {
        self.active_player = None;

        let sessions = self.session_manager.GetSessions().expect("No sessions?");
        let preferred = preferred.unwrap_or_default();
        for session in sessions {
            if (session.SourceAppUserModelId().unwrap().to_string() == preferred)
                || session.GetPlaybackInfo().unwrap().PlaybackStatus().unwrap()
                    == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
            {
                self.active_player = Some(session.SourceAppUserModelId().unwrap().to_string());
                break;
            }
        }
    }

    async fn get_session_capabilities(session: GlobalSystemMediaTransportControlsSession) -> Capabilities{
	    let controls = session.GetPlaybackInfo().unwrap().Controls().unwrap();

        let mut capabilities = Capabilities {
            can_play_pause: controls.IsPlayEnabled().unwrap_or(false) || controls.IsPauseEnabled().unwrap_or(false),
            can_go_next: controls.IsNextEnabled().unwrap_or(false),
            can_go_previous: controls.IsPreviousEnabled().unwrap_or(false),
            can_seek: controls.IsPlaybackPositionEnabled().unwrap_or(false) && session.GetTimelineProperties().unwrap().EndTime().unwrap_or_default().Duration != 0,
            can_control: false
        };
	    capabilities.can_control =
            capabilities.can_play_pause ||
            capabilities.can_go_next ||
            capabilities.can_go_previous ||
            capabilities.can_seek;
        
        capabilities
    }

    pub async fn play(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No player"),
            Some(session) => match session.TryPlayAsync() {
                Ok(result) => Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    pub async fn pause(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No player"),
            Some(session) => match session.TryPauseAsync() {
                Ok(result) => Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    pub async fn play_pause(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No player"),
            Some(session) => match session.TryTogglePlayPauseAsync() {
                Ok(result) => Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    pub async fn stop(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No player"),
            Some(session) => match session.TryStopAsync() {
                Ok(result) => Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    pub async fn next(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No player"),
            Some(session) => match session.TrySkipNextAsync() {
                Ok(result) => Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    pub async fn previous(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No player"),
            Some(session) => match session.TrySkipPreviousAsync() {
                Ok(result) => Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    pub async fn shuffle(&self) -> Result<bool, &str> {
        match self.get_session() {
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

    pub async fn repeat(&self) -> Result<bool, &str> {
        match self.get_session() {
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

    pub async fn seek(&self, offset_us: i64) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No player"),
            Some(session) => match session.GetTimelineProperties() {
                Ok(timeline_properties) => {
                    let position = timeline_properties.Position().unwrap_or_default();
                    self.set_position((position.Duration + offset_us) as f64 / 1000f64).await
                }
                Err(_) => Err("Could not get timeline properties"),
            },
        }
    }

    pub async fn seek_percentage(&self, percentage: f64) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No player"),
            Some(session) => match session.GetTimelineProperties() {
                Ok(timeline_properties) => {
                    let start_time = timeline_properties.StartTime().unwrap_or_default();
                    let end_time = timeline_properties.EndTime().unwrap_or_default();
                    let length = (end_time.Duration - start_time.Duration) as f64 / 1000.0;
                    self.set_position(length * percentage).await
                }
                Err(_) => Err("Could not get timeline properties"),
            },
        }
    }

    pub async fn set_position(&self, position_s: f64) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No player"),
            Some(session) => {
                match session.TryChangePlaybackPositionAsync((position_s * 1000.0) as i64) {
                    // probabilmente non worka e la pos sara' wonky
                    Ok(result) => Ok(result.await.unwrap_or(false)),
                    Err(_) => Err("Error while trying to perform the command"),
                }
            }
        }
    }

    pub async fn get_position(&self) -> Result<f64, &str> {
        match self.get_session() {
            None => Err("No player"),
            Some(session) => match session.GetTimelineProperties() {
                Ok(timeline_properties) => {
                    if timeline_properties.EndTime().unwrap_or_default().Duration == 0 {
                        return Ok(0f64);
                    }

                    let mut position = timeline_properties.Position().unwrap_or_default().Duration;
                    let playback_status =
                        session.GetPlaybackInfo().unwrap().PlaybackStatus().unwrap();

                    if playback_status
                        == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
                    {
                        let time_from_last_update = chrono::offset::Utc::now().timestamp_millis()
                            - timeline_properties
                                .LastUpdatedTime()
                                .unwrap_or(Foundation::DateTime { UniversalTime: 0 })
                                .UniversalTime;
                        position += time_from_last_update;
                    }

                    Ok(
                        (position - timeline_properties.StartTime().unwrap().Duration) as f64 / 1000f64,
                    )
                }
                Err(_) => Err("Could not get timeline properties"),
            },
        }
    }
}
