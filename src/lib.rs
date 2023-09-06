use jsplayermanager::JsPlayerManager;
use napi_derive::napi;
use owo::{playermanager::PlayerManager, util::get_session_player_name};

mod owo;

mod jsplayer;
mod jsplayermanager;

#[napi]
pub async fn get_manager() -> JsPlayerManager {
    let mut player_manager = PlayerManager::new().await.unwrap();
    let rx = player_manager.set_events();
    JsPlayerManager {
        player_manager,
        rx
    }
}

#[napi]
pub async fn get_friendly_name_for(aumid: String) -> Option<String> {
    get_session_player_name(&aumid).await
}
