use napi_derive::napi;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::jsplayer::JsPlayer;
use crate::owo::playermanager::{PlayerManager, ManagerEvent};

#[napi(js_name = "PlayerManager")]
pub struct JsPlayerManager {
    player_manager: PlayerManager,
    rx: UnboundedReceiver<ManagerEvent>
}

#[napi]
impl JsPlayerManager {
    #[napi]
    pub async unsafe fn poll_next_event(&mut self) -> String {
        match self.rx.recv().await.unwrap() {
            ManagerEvent::CurrentSessionChanged => String::from("CurrentSessionChanged"),
            ManagerEvent::SessionsChanged => String::from("SessionsChanged")
        }
    }

    #[napi]
    pub fn get_active_session(&self) -> Option<JsPlayer> {
        if let Some(player) = self.player_manager.get_active_session() {
            return Some(JsPlayer::wrap_player(player));
        }
        None
    }

    #[napi]
    pub fn get_session(&self, aumid: String) -> Option<JsPlayer> {
        if let Some(player) = self.player_manager.get_session(&aumid) {
            return Some(JsPlayer::wrap_player(player));
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
            return Some(JsPlayer::wrap_player(player)); // TECNICAMENTE come lo prendiamo l'owned?
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
