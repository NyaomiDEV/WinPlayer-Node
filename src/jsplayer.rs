use std::sync::Arc;

use napi_derive::napi;
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};
use windows::Media::MediaPlaybackAutoRepeatMode;

use crate::owo::{
    player::{Player, PlayerEvent},
    types::{Position, Status},
};

#[napi(js_name = "Player")]
pub struct JsPlayer {
    player: Arc<Mutex<Player>>,
    rx: UnboundedReceiver<PlayerEvent>,
}

#[napi]
impl JsPlayer {
    pub async fn wrap_player(player: Arc<Mutex<Player>>) -> Self {
        let rx = player.lock().await.set_events();
        JsPlayer { player, rx }
    }

    #[napi]
    pub async unsafe fn poll_next_event(&mut self) -> String {
        match self.rx.recv().await.unwrap() {
            PlayerEvent::PlaybackInfoChanged => String::from("PlaybackInfoChanged"),
            PlayerEvent::MediaPropertiesChanged => String::from("MediaPropertiesChanged"),
            PlayerEvent::TimelinePropertiesChanged => String::from("TimelinePropertiesChanged"),
        }
    }

    #[napi]
    pub async fn get_status(&self) -> Status {
        self.player.lock().await.get_status().await
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
        let _value = match value.as_str() {
            "None" => MediaPlaybackAutoRepeatMode::None,
            "List" => MediaPlaybackAutoRepeatMode::List,
            "Track" => MediaPlaybackAutoRepeatMode::Track,
            _ => MediaPlaybackAutoRepeatMode::None,
        };
        self.player.lock().await.set_repeat(_value).await
    }

    #[napi]
    pub async fn get_repeat(&self) -> Option<String> {
        if let Some(repeat_mode) = self.player.lock().await.get_repeat() {
            return Some(match repeat_mode {
                MediaPlaybackAutoRepeatMode::None => String::from("None"),
                MediaPlaybackAutoRepeatMode::List => String::from("List"),
                MediaPlaybackAutoRepeatMode::Track => String::from("Track"),
                _ => String::from("None"),
            });
        }
        None
    }

    #[napi]
    pub async fn seek(&self, offset_us: i64) -> bool {
        self.player.lock().await.seek(offset_us).await
    }

    #[napi]
    pub async fn seek_percentage(&self, percentage: f64) -> bool {
        self.player.lock().await.seek_percentage(percentage).await
    }

    #[napi]
    pub async fn set_position(&self, position_s: f64) -> bool {
        self.player.lock().await.set_position(position_s).await
    }

    #[napi]
    pub async fn get_position(&self, wants_current_position: bool) -> Option<Position> {
        self.player
            .lock()
            .await
            .get_position(wants_current_position)
            .await
    }
}
