use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{self},
    Mutex,
};

use windows::{
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
};

use crate::player::Player;

enum ManagerEvent {
    SessionsChanged,
    CurrentSessionChanged,
}

struct EventToken {
    sessions_changed_token: EventRegistrationToken,
    current_session_changed_token: EventRegistrationToken,
}
pub struct PlayerManager {
    denylist: Option<Vec<String>>,
    session_manager: GlobalSystemMediaTransportControlsSessionManager,
    active_player_key: Option<String>, // che volevo storare una ref ma mi rompe il cazzo con le lifetimes
    system_player_key: Option<String>,

    event_tokens: Option<EventToken>,
    players: HashMap<String, Player>,
}

impl PlayerManager {
    pub async fn new(denylist: Option<Vec<String>>) -> Option<Arc<Mutex<Self>>> {
        if let Ok(_binding) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
            if let Ok(session_manager) = _binding.await {
                // Registra eventi nel session manager QUI
                let (loop_tx, mut loop_rx) = mpsc::unbounded_channel();

                let player_manager = Arc::new(Mutex::new(PlayerManager {
                    denylist,
                    session_manager,
                    active_player_key: None,
                    system_player_key: None,
                    event_tokens: None,
                    players: HashMap::new(),
                }));

                let s = player_manager.clone();

                tokio::task::spawn(async move {
                    loop {
                        match loop_rx.recv().await {
                            Some(ManagerEvent::CurrentSessionChanged) => {
                                s.lock().await.update_system_session();
                            }
                            Some(ManagerEvent::SessionsChanged) => {
                                let preferred = s.lock().await.active_player_key.clone();
                                let denylist = s.lock().await.denylist.clone();
                                s.lock()
                                    .await
                                    .update_sessions(preferred.as_ref(), denylist.as_ref());
                            }
                            None => {}
                        }
                    }
                });

                // Register SessionsChanged handle
                let sessions_changed_handler = TypedEventHandler::new({
                    let tx = loop_tx.clone();
                    move |_, _| {
                        let _ = tx.send(ManagerEvent::SessionsChanged);
                        Ok(())
                    }
                });

                let current_session_changed_handler = TypedEventHandler::new({
                    let tx = loop_tx.clone();
                    move |_, _| {
                        let _ = tx.send(ManagerEvent::CurrentSessionChanged);
                        Ok(())
                    }
                });

                let sessions_changed_token = player_manager
                    .lock()
                    .await
                    .session_manager
                    .SessionsChanged(&sessions_changed_handler)
                    .unwrap();
                let current_session_changed_token = player_manager
                    .lock()
                    .await
                    .session_manager
                    .CurrentSessionChanged(&current_session_changed_handler)
                    .unwrap();

                player_manager.lock().await.event_tokens = Some(EventToken {
                    sessions_changed_token,
                    current_session_changed_token,
                });

                let _ = loop_tx.send(ManagerEvent::SessionsChanged);

                return Some(player_manager);
            }
        }
        None
    }

    pub fn get_session(&self) -> Option<&Player> {
        if let Some(player_key) = &self.active_player_key {
            return self.players.get(player_key);
        }
        None
    }

    pub fn get_system_session(&self) -> Option<&Player> {
        if let Some(player_key) = &self.system_player_key {
            return self.players.get(player_key);
        }
        None
    }

    fn update_system_session(&mut self) {
        if let Ok(session) = self.session_manager.GetCurrentSession() {
            self.system_player_key = None;

            if let Ok(aumid) = session.SourceAppUserModelId() {
                let _aumid = aumid.to_string();
                if _aumid.is_empty() {
                    return;
                }

                self.system_player_key = Some(_aumid);
            }
        }
    }

    fn update_sessions(&mut self, preferred: Option<&String>, denylist: Option<&Vec<String>>) {
        if let Ok(sessions) = self.session_manager.GetSessions() {
            self.active_player_key = None;
            for session in sessions {
                if let Ok(aumid) = session.SourceAppUserModelId() {
                    let _aumid = aumid.to_string();
                    if _aumid.is_empty() {
                        continue;
                    }

                    if denylist.is_some_and(|x| x.contains(&_aumid)) {
                        continue;
                    }

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
                            if result.eq(&_aumid) {
                                break 'rt true;
                            }
                        }
                        false
                    };

                    if !self.players.contains_key(&_aumid) {
                        let player = Player::new(session, _aumid.clone());
                        self.players.insert(_aumid.clone(), player);
                    }

                    if is_preferred
                        || playback_status
                            == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
                    {
                        self.active_player_key = Some(_aumid);
                        break;
                    }
                }
            }
        }
    }
}

impl Drop for PlayerManager {
    fn drop(&mut self) {
        let _ = self
            .session_manager
            .RemoveSessionsChanged(self.event_tokens.as_mut().unwrap().sessions_changed_token);
        let _ = self.session_manager.RemoveCurrentSessionChanged(
            self.event_tokens
                .as_mut()
                .unwrap()
                .current_session_changed_token,
        );
    }
}
