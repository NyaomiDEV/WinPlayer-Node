use std::{sync::{Arc, Mutex}, collections::HashMap};

use windows::{Media::Control::{GlobalSystemMediaTransportControlsSessionManager, GlobalSystemMediaTransportControlsSessionPlaybackStatus}, Foundation::TypedEventHandler};

use crate::player::Player;

struct PlayerManager {
    denylist: Option<Vec<String>>,
    session_manager: GlobalSystemMediaTransportControlsSessionManager,
    active_player: Option<&Player>, // cm si fixa sks
	players: HashMap<String, Player>,
}

impl PlayerManager {
    pub async fn new(denylist: Option<Vec<String>>) -> Self {
        let session_manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .expect("The session manager is kil")
            .await
            .expect("The session manager is kil 2");

        PlayerManager {
            denylist,
            session_manager,
            active_player: None,
			players: HashMap::new(),
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
                    let preferred = s.active_player;
                    let denylist = s.denylist.clone();
                    s.update_sessions(preferred, denylist.as_ref());
                })
            }
        });

        rc_self
            .lock()
            .unwrap()
            .session_manager
            .SessionsChanged(&handler);

        let preferred = rc_self.lock().unwrap().active_player;
        let denylist = rc_self.lock().unwrap().denylist.clone();
        rc_self
            .lock()
            .unwrap()
            .update_sessions(preferred, denylist.as_ref());
    }

    fn get_session(&self) -> Option<&Player> {
        return self.active_player;
    }

    fn update_sessions(
        &mut self,
        preferred: Option<&Player>,
        denylist: Option<&Vec<String>>,
    ) {
        if let Ok(sessions) = self.session_manager.GetSessions() {
            self.active_player = None;

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
                            if session == result.session {
                                break 'rt true;
                            }
                        }
                        false
                    };
					
					if ! self.players.contains_key(&_aumid) {
						let player = Player::new(session);
						self.players.insert(_aumid, player);
					}

					let player_ref = self.players.get(&_aumid).unwrap();

                    if is_preferred
                        || playback_status
                            == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
                    {
                        self.active_player = Some(player_ref);
                        break;
                    }
                }
            }
        }
    }	
}