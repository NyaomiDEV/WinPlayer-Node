use std::sync::{Arc, Mutex};

use windows::{
    Foundation::TypedEventHandler,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Media::MediaPlaybackAutoRepeatMode,
};

use crate::types::{Position, Update};

use crate::util::{
    compute_position, get_session_status,
};

struct Player {
    session_manager: GlobalSystemMediaTransportControlsSessionManager,
    active_player: Option<String>,
    denylist: Option<Vec<String>>,
}

impl Player {
    pub async fn new(denylist: Option<Vec<String>>) -> Self {
        let session_manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .expect("The session manager is kil")
            .await
            .expect("The session manager is kil 2");

        Player {
            session_manager,
            active_player: None,
            denylist,
        }
    }

    pub fn run(self) {
        // Possiamo autostartarla dal costruttore o integrarla a esso?
        // Passando self così non rischiamo di perdercelo dopo questa call?
        let rc_self = Arc::new(Mutex::new(self));

        let handler = TypedEventHandler::new({
            let s = rc_self.clone();
            move |_, _| {
                Ok({
                    let mut binding = s.lock();
                    let s = binding.as_mut().unwrap();
                    let preferred = s.active_player.clone();
                    let denylist = s.denylist.clone();
                    s.update_sessions(preferred.as_ref(), denylist.as_ref());
                })
            }
        });

        rc_self
            .lock()
            .unwrap()
            .session_manager
            .SessionsChanged(&handler);

        let preferred = rc_self.lock().unwrap().active_player.clone();
        let denylist = rc_self.lock().unwrap().denylist.clone();
        rc_self
            .lock()
            .unwrap()
            .update_sessions(preferred.as_ref(), denylist.as_ref());
    }

    fn get_session(&self) -> Option<GlobalSystemMediaTransportControlsSession> {
        if let Ok(sessions) = self.session_manager.GetSessions() {
            if let Some(active_player) = &self.active_player {
                for session in sessions {
                    if let Ok(aumid) = session.SourceAppUserModelId() {
                        let _aumid = aumid.to_string();
                        if _aumid.eq(active_player) {
                            return Some(session);
                        }
                    }
                }
            }
        }
        None
    }

    fn update_sessions(
        &mut self,
        preferred: Option<&String>,
        denylist: Option<&Vec<String>>,
    ) {
        if let Ok(sessions) = self.session_manager.GetSessions() {
            self.active_player = None;

            for session in sessions {
                if let Ok(aumid) = session.SourceAppUserModelId() {
                    let _aumid = aumid.to_string();
                    if denylist.is_some_and(|x| x.contains(&_aumid)) {
                        continue;
                    }

                    // Questo player viene preso in considerazione, bisogna registrare gli eventi
                    // this.register_session_events(session)
                    // Bisogna conservarne l'AUMID in un vettore

                    let playback_status = 'rt: {
                        if let Ok(playback_info) = session.GetPlaybackInfo() {
                            if let Ok(playback_status) = playback_info.PlaybackStatus() {
                                break 'rt playback_status;
                            }
                        }
                        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped
                    };

                    let is_preferred = 'rt: {
                        if let Some(result) = preferred {
                            if _aumid.eq(result) {
                                break 'rt true;
                            }
                        }
                        false
                    };

                    if is_preferred
                        || playback_status
                            == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
                    {
                        self.active_player = Some(_aumid);
                        break;
                    }
                }
            }

            // Qui bisognerebbe iterare sugli AUMID ancora presenti nel vettore
            // e deregistrare tutti gli eventi che sono stati registrati su sessioni ormai perse (non presenti).
            // Ma possiamo revocare gli handler su una sessione che è persa?
            // Dovremmo "salvarci" i puntatori alle sessioni su una HashMap stile vecchio progetto?
            // Cosa prevede Windows-RS?
            // Ad esempio su C++/WinRT sta un token "winrt::auto_revoke_t" ma non so qua
        }
    }

    pub async fn get_active_session_status(&self) -> Option<Update> {
        if let Some(session) = self.get_session() {
            return Some(get_session_status(session).await);
        }
        None
    }

    pub async fn play(&self) -> bool {
        if let Some(session) = self.get_session() {
            if let Ok(result) = session.TryPlayAsync() {
                return result.await.unwrap_or(false);
            }
        }
        false
    }

    pub async fn pause(&self) -> bool {
        if let Some(session) = self.get_session() {
            if let Ok(result) = session.TryPauseAsync() {
                return result.await.unwrap_or(false);
            }
        }
        false
    }

    pub async fn play_pause(&self) -> bool {
        if let Some(session) = self.get_session() {
            if let Ok(result) = session.TryTogglePlayPauseAsync() {
                return result.await.unwrap_or(false);
            }
        }
        false
    }

    pub async fn stop(&self) -> bool {
        if let Some(session) = self.get_session() {
            if let Ok(result) = session.TryStopAsync() {
                return result.await.unwrap_or(false);
            }
        }
        false
    }

    pub async fn next(&self) -> bool {
        if let Some(session) = self.get_session() {
            if let Ok(result) = session.TrySkipNextAsync() {
                return result.await.unwrap_or(false);
            }
        }
        false
    }

    pub async fn previous(&self) -> bool {
        if let Some(session) = self.get_session() {
            if let Ok(result) = session.TrySkipPreviousAsync() {
                return result.await.unwrap_or(false);
            }
        }
        false
    }

    pub async fn shuffle(&self) -> bool {
        if let Some(session) = self.get_session() {
            if let Ok(playback_info) = session.GetPlaybackInfo() {
                if let Ok(shuffle_active) = playback_info.IsShuffleActive() {
                    if let Ok(result) =
                        session.TryChangeShuffleActiveAsync(shuffle_active.Value().unwrap_or(false))
                    {
                        return result.await.unwrap_or(false);
                    }
                }
            }
        }
        false
    }

    pub async fn repeat(&self) -> bool {
        if let Some(session) = self.get_session() {
            if let Ok(playback_info) = session.GetPlaybackInfo() {
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
                    if let Ok(result) = session.TryChangeAutoRepeatModeAsync(new_repeat_mode) {
                        return result.await.unwrap_or(false);
                    }
                }
            }
        }
        false
    }

    pub async fn seek(&self, offset_us: i64) -> bool {
        if let Some(session) = self.get_session() {
            if let Ok(timeline_properties) = session.GetTimelineProperties() {
                if let Ok(position) = timeline_properties.Position() {
                    return self
                        .set_position((position.Duration + offset_us) as f64 / 1000f64)
                        .await;
                }
            }
        }
        false
    }

    pub async fn seek_percentage(&self, percentage: f64) -> bool {
        if let Some(session) = self.get_session() {
            if let Ok(timeline_properties) = session.GetTimelineProperties() {
                let start_time = timeline_properties.StartTime().unwrap_or_default();
                let end_time = timeline_properties.EndTime().unwrap_or_default();
                let length = (end_time.Duration - start_time.Duration) as f64 / 1000.0;
                return self.set_position(length * percentage).await;
            }
        }
        false
    }

    pub async fn set_position(&self, position_s: f64) -> bool {
        if let Some(session) = self.get_session() {
            if let Ok(result) = session.TryChangePlaybackPositionAsync((position_s * 1000.0) as i64)
            {
                // probabilmente non worka e la pos sara' wonky
                return result.await.unwrap_or(false);
            }
        }
        false
    }

    pub async fn get_position(&self) -> Option<Position> {
        if let Some(session) = self.get_session() {
            return compute_position(
                session.GetTimelineProperties().ok().as_ref(),
                session.GetPlaybackInfo().ok().as_ref(),
                true,
            );
        }
        None
    }
}
