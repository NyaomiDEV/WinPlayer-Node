use jsplayermanager::JsPlayerManager;
use napi_derive::napi;
use owo::playermanager::PlayerManager;

mod owo;

mod jsplayer;
mod jsplayermanager;

#[napi]
pub async fn get_manager() -> JsPlayerManager {
	JsPlayerManager {
		player_manager: PlayerManager::new().await.unwrap(),
		event_callback_tsfn: None
	}
}