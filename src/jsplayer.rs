use std::sync::Arc;

use napi::bindgen_prelude::External;
use napi_derive::napi;
use tokio::sync::Mutex;

use crate::{
    jstypes::{JsPosition, JsStatus},
    owo::player::{Player, PlayerEvent},
};

#[napi(js_name = "Player")]
pub struct JsPlayer {
    player: External<Arc<Mutex<Player>>>,
}

#[napi]
impl JsPlayer {
    #[napi(constructor)]
    pub fn new(player: External<Arc<Mutex<Player>>>) -> Self {
        JsPlayer { player }
    }

    #[napi]
    pub async unsafe fn poll_next_event(&mut self) -> String {
        match self.player.lock().await.poll_next_event().await {
            Some(PlayerEvent::PlaybackInfoChanged) => String::from("PlaybackInfoChanged"),
            Some(PlayerEvent::MediaPropertiesChanged) => String::from("MediaPropertiesChanged"),
            Some(PlayerEvent::TimelinePropertiesChanged) => {
                String::from("TimelinePropertiesChanged")
            }
            None => String::from("None"),
        }
    }

    #[napi(ts_return_type = "Promise<Status>")]
    pub async fn get_status(&self) -> JsStatus {
        JsStatus::from(self.player.lock().await.get_status().await)
    }

    #[napi]
    pub async fn get_aumid(&self) -> String {
        self.player.lock().await.get_aumid()
    }

    #[napi]
    pub async fn play(&self) -> bool {
        self.player.lock().await.play().await
    }

    #[napi]
    pub async fn pause(&self) -> bool {
        self.player.lock().await.pause().await
    }

    #[napi]
    pub async fn play_pause(&self) -> bool {
        self.player.lock().await.play_pause().await
    }

    #[napi]
    pub async fn stop(&self) -> bool {
        self.player.lock().await.stop().await
    }

    #[napi]
    pub async fn get_playback_status(&self) -> String {
        self.player.lock().await.get_playback_status()
    }

    #[napi]
    pub async fn next(&self) -> bool {
        self.player.lock().await.next().await
    }

    #[napi]
    pub async fn previous(&self) -> bool {
        self.player.lock().await.previous().await
    }

    #[napi]
    pub async fn set_shuffle(&self, value: bool) -> bool {
        self.player.lock().await.set_shuffle(value).await
    }

    #[napi]
    pub async fn get_shuffle(&self) -> bool {
        self.player.lock().await.get_shuffle()
    }

    #[napi]
    pub async fn set_repeat(&self, value: String) -> bool {
        self.player.lock().await.set_repeat(value).await
    }

    #[napi]
    pub async fn get_repeat(&self) -> String {
        self.player.lock().await.get_repeat()
    }

    #[napi]
    pub async fn seek(&self, offset_s: f64) -> bool {
        self.player.lock().await.seek(offset_s).await
    }

    #[napi]
    pub async fn seek_percentage(&self, percentage: f64) -> bool {
        self.player.lock().await.seek_percentage(percentage).await
    }

    #[napi]
    pub async fn set_position(&self, position_s: f64) -> bool {
        self.player.lock().await.set_position(position_s).await
    }

    #[napi(ts_return_type = "Promise<Position | null>")]
    pub async fn get_position(&self, wants_current_position: bool) -> Option<JsPosition> {
        if let Some(position) = self
            .player
            .lock()
            .await
            .get_position(wants_current_position)
            .await
        {
            return Some(JsPosition::from(position));
        }
        None
    }
}
