use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;

use windows::{
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
};

use crate::owo::{player::Player, types::CallbackFn};

enum ManagerEvent {
    SessionsChanged,
    CurrentSessionChanged,
}

struct EventToken {
    sessions_changed_token: EventRegistrationToken,
    current_session_changed_token: EventRegistrationToken,
}
pub struct PlayerManager {
    session_manager: GlobalSystemMediaTransportControlsSessionManager,

    active_player_key: Option<String>,
    system_player_key: Option<String>,
    players: HashMap<String, Arc<Player>>,

    event_tokens: Option<EventToken>,
}

impl PlayerManager {
    pub async fn new() -> Option<Self> {
        if let Ok(_binding) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
            if let Ok(session_manager) = _binding.await {
                return Some(PlayerManager {
                    session_manager,

                    players: HashMap::new(),
                    active_player_key: None,
                    system_player_key: None,

                    event_tokens: None,
                });
            }
        }
        None
    }

    pub fn set_event_callback(&mut self, callback: Box<CallbackFn>) {
        let (tx, mut rx) = mpsc::unbounded_channel();

        tokio::task::spawn(async move {
            loop {
                match rx.recv().await {
                    Some(ManagerEvent::CurrentSessionChanged) => {
                        callback(String::from("CurrentSessionChanged"))
                    }
                    Some(ManagerEvent::SessionsChanged) => {
                        callback(String::from("SessionsChanged"))
                    }
                    None => {}
                }
            }
        });

        let sessions_changed_handler = TypedEventHandler::new({
            let tx = tx.clone();
            move |_, _| {
                let _ = tx.send(ManagerEvent::SessionsChanged);
                Ok(())
            }
        });

        let current_session_changed_handler = TypedEventHandler::new({
            let tx = tx.clone();
            move |_, _| {
                let _ = tx.send(ManagerEvent::CurrentSessionChanged);
                Ok(())
            }
        });

        let sessions_changed_token = self
            .session_manager
            .SessionsChanged(&sessions_changed_handler)
            .unwrap();
        let current_session_changed_token = self
            .session_manager
            .CurrentSessionChanged(&current_session_changed_handler)
            .unwrap();

        self.event_tokens = Some(EventToken {
            sessions_changed_token,
            current_session_changed_token,
        });

        let _ = tx.send(ManagerEvent::SessionsChanged);
    }

    pub fn unset_event_callback(&mut self) {
        let _ = self
            .session_manager
            .RemoveSessionsChanged(self.event_tokens.as_mut().unwrap().sessions_changed_token);
        let _ = self.session_manager.RemoveCurrentSessionChanged(
            self.event_tokens
                .as_mut()
                .unwrap()
                .current_session_changed_token,
        );

        self.event_tokens = None;
    }

    pub fn get_active_session(&self) -> Option<Arc<Player>> {
        if let Some(player_key) = &self.active_player_key {
            return Some(self.players.get(player_key)?.clone());
        }
        None
    }

    pub fn get_session(&self, aumid: &String) -> Option<Arc<Player>> {
        Some(self.players.get(aumid)?.clone())
    }

    pub fn get_sessions_keys(&self) -> Vec<String> {
        self.players
            .keys()
            .map(String::from)
            .collect::<Vec<String>>()
    }

    pub fn get_system_session(&self) -> Option<Arc<Player>> {
        if let Some(player_key) = &self.system_player_key {
            return Some(self.players.get(player_key)?.clone());
        }
        None
    }

    pub fn update_system_session(&mut self) {
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

    pub fn update_sessions(&mut self, denylist: Option<&Vec<String>>) {
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
                        if let Some(result) = self.active_player_key.clone() {
                            if result.eq(&_aumid) {
                                break 'rt true;
                            }
                        }
                        false
                    };

                    if !self.players.contains_key(&_aumid) {
                        let player = Arc::new(Player::new(session, _aumid.clone()));
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
        self.unset_event_callback();
    }
}
