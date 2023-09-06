use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;

use crate::jsplayer::JsPlayer;
use crate::owo::playermanager::PlayerManager;

#[napi(js_name = "PlayerManager")]
pub struct JsPlayerManager {
    player_manager: PlayerManager,
    event_callback_tsfn: Option<ThreadsafeFunction<Vec<String>, ErrorStrategy::Fatal>>,
}

#[napi]
impl JsPlayerManager {
    #[napi]
    pub fn set_event_callback(&mut self, callback: napi::JsFunction) {
        self.event_callback_tsfn = Some(
            callback
                .create_threadsafe_function(0, |ctx| Ok(ctx.value))
                .unwrap(),
        );

        let event_callback = Box::new({
            let _fn = self.event_callback_tsfn.clone();
            |event: String| {
                if let Some(tsfn) = _fn {
                    tsfn.call(vec![event], ThreadsafeFunctionCallMode::NonBlocking);
                }
            }
        });

        self.player_manager.set_event_callback(event_callback);
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
