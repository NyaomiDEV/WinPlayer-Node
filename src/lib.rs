use jsplayermanager::JsPlayerManager;
use napi::bindgen_prelude::External;
use napi_derive::napi;
use owo::{util::get_session_player_name, playermanager::PlayerManager};

mod owo;

mod jsplayer;
mod jsplayermanager;
mod jstypes;

#[napi]
pub async fn get_player_manager() -> JsPlayerManager {
    let player_manager = PlayerManager::new().await.unwrap();
    JsPlayerManager::new(External::new(player_manager))
}

#[napi]
pub async fn get_friendly_name_for(aumid: String) -> Option<String> {
    get_session_player_name(&aumid).await
}
