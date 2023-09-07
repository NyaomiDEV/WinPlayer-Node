use jsplayermanager::JsPlayerManager;
use napi_derive::napi;
use owo::util::get_session_player_name;

mod owo;

mod jsplayer;
mod jsplayermanager;
mod jstypes;

#[napi]
pub async fn get_player_manager() -> JsPlayerManager {
    JsPlayerManager::get().await
}

#[napi]
pub async fn get_friendly_name_for(aumid: String) -> Option<String> {
    get_session_player_name(&aumid).await
}
