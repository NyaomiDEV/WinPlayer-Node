use napi::bindgen_prelude::External;
use napi_derive::napi;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::jsplayer::JsPlayer;
use crate::owo::playermanager::{ManagerEvent, PlayerManager};

#[napi(js_name = "PlayerManager")]
pub struct JsPlayerManager {
    internal: External<JsPlayerManagerInternal>,
}

pub struct JsPlayerManagerInternal {
    pub(crate) player_manager: PlayerManager,
    pub(crate) rx: UnboundedReceiver<ManagerEvent>,
}

#[napi]
impl JsPlayerManager {
    #[napi(constructor)]
    pub fn new(internal: External<JsPlayerManagerInternal>) -> Self {
        JsPlayerManager { internal }
    }

    pub async fn get() -> Self {
        let mut player_manager = PlayerManager::new().await.unwrap();
        let rx = player_manager.set_events();
        let internal = JsPlayerManagerInternal { player_manager, rx };
        JsPlayerManager::new(External::new(internal))
    }

    #[napi]
    pub async unsafe fn poll_next_event(&mut self) -> String {
        match self.internal.rx.recv().await.unwrap() {
            ManagerEvent::ActiveSessionChanged => String::from("ActiveSessionChanged"),
            ManagerEvent::SystemSessionChanged => String::from("SystemSessionChanged"),
            ManagerEvent::SessionsChanged => String::from("SessionsChanged"),
        }
    }

    #[napi]
    pub async fn get_active_session(&self) -> Option<JsPlayer> {
        if let Some(player) = self.internal.player_manager.get_active_session() {
            return Some(JsPlayer::wrap_player(player).await);
        }
        None
    }

    #[napi]
    pub async fn get_session(&self, aumid: String) -> Option<JsPlayer> {
        if let Some(player) = self.internal.player_manager.get_session(&aumid) {
            return Some(JsPlayer::wrap_player(player).await);
        }
        None
    }

    #[napi]
    pub fn get_sessions_keys(&self) -> Vec<String> {
        self.internal.player_manager.get_sessions_keys()
    }

    #[napi]
    pub async fn get_system_session(&self) -> Option<JsPlayer> {
        if let Some(player) = self.internal.player_manager.get_system_session() {
            return Some(JsPlayer::wrap_player(player).await);
        }
        None
    }

    #[napi]
    pub fn update_system_session(&mut self) {
        self.internal.player_manager.update_system_session()
    }

    #[napi]
    pub fn update_sessions(&mut self, denylist: Option<Vec<String>>) {
        self.internal
            .player_manager
            .update_sessions(denylist.as_ref())
    }
}
