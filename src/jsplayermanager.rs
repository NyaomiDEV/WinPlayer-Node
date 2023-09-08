use napi::bindgen_prelude::External;
use napi_derive::napi;

use crate::jsplayer::JsPlayer;
use crate::owo::playermanager::{ManagerEvent, PlayerManager};

#[napi(js_name = "PlayerManager")]
pub struct JsPlayerManager {
    player_manager: External<PlayerManager>,
}

#[napi]
impl JsPlayerManager {
    #[napi(constructor)]
    pub fn new(player_manager: External<PlayerManager>) -> Self {
        JsPlayerManager { player_manager }
    }

    #[napi]
    pub async unsafe fn poll_next_event(&mut self) -> String {
        match self.player_manager.poll_next_event().await {
            Some(ManagerEvent::ActiveSessionChanged) => String::from("ActiveSessionChanged"),
            Some(ManagerEvent::SystemSessionChanged) => String::from("SystemSessionChanged"),
            Some(ManagerEvent::SessionsChanged) => String::from("SessionsChanged"),
            None => String::from("None"),
        }
    }

    #[napi]
    pub fn get_active_session(&self) -> Option<JsPlayer> {
        if let Some(player) = self.player_manager.get_active_session() {
            return Some(JsPlayer::new(External::new(player)));
        }
        None
    }

    #[napi]
    pub fn get_session(&self, aumid: String) -> Option<JsPlayer> {
        if let Some(player) = self.player_manager.get_session(&aumid) {
            return Some(JsPlayer::new(External::new(player)));
        }
        None
    }

    #[napi]
    pub fn get_sessions_keys(&self) -> Vec<String> {
        self.player_manager.get_sessions_keys()
    }

    #[napi]
    pub fn get_system_session(&self) -> Option<JsPlayer> {
        if let Some(player) = self.player_manager.get_system_session() {
            return Some(JsPlayer::new(External::new(player)));
        }
        None
    }

    #[napi]
    pub fn update_system_session(&mut self) {
        self.player_manager.update_system_session()
    }

    #[napi]
    pub fn update_sessions(&mut self, denylist: Option<Vec<String>>) {
        self.player_manager.update_sessions(denylist.as_ref())
    }
}
