use jsplayermanager::JsPlayerManager;
use napi::bindgen_prelude::External;
use napi_derive::napi;
use owo::{playermanager::PlayerManager, util::get_session_player_name};

mod owo;

mod jsplayer;
mod jsplayermanager;
mod jstypes;

#[napi]
pub async fn get_player_manager() -> Option<JsPlayerManager> {
    if let Some(player_manager) = PlayerManager::new().await {
        return Some(JsPlayerManager::new(External::new(player_manager)));
    }
    None
}

#[napi]
pub async fn get_friendly_name_for(aumid: String) -> Option<String> {
    get_session_player_name(&aumid).await
}
