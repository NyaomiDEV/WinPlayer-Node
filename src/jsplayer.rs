use std::sync::Arc;

use napi_derive::napi;
use tokio::sync::mpsc::UnboundedReceiver;
use windows::Media::MediaPlaybackAutoRepeatMode;

use crate::owo::{
    player::{Player, PlayerEvent},
    types::{Position, Status},
};

#[napi(js_name = "Player")]
pub struct JsPlayer {
    player: Arc<Player>,
    rx: UnboundedReceiver<PlayerEvent>
}

#[napi]
impl JsPlayer {
    pub fn wrap_player(player: Arc<Player>) -> Self {
        let rx = player.set_events(); // eh, bello l'arc
        JsPlayer {
            player,
            rx
        }
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
    pub async fn get_session_status(&self) -> Status {
        self.player.get_session_status().await
    }

    #[napi]
    pub fn get_aumid(&self) -> String {
        self.player.get_aumid()
    }

    #[napi]
    pub async fn play(&self) -> bool {
        self.player.play().await
    }

    #[napi]
    pub async fn pause(&self) -> bool {
        self.player.pause().await
    }

    #[napi]
    pub async fn play_pause(&self) -> bool {
        self.player.play_pause().await
    }

    #[napi]
    pub async fn stop(&self) -> bool {
        self.player.stop().await
    }

    #[napi]
    pub async fn next(&self) -> bool {
        self.player.next().await
    }

    #[napi]
    pub async fn previous(&self) -> bool {
        self.player.previous().await
    }

    #[napi]
    pub async fn set_shuffle(&self, value: bool) -> bool {
        self.player.set_shuffle(value).await
    }

    #[napi]
    pub fn get_shuffle(&self) -> bool {
        self.player.get_shuffle()
    }

    #[napi]
    pub async fn set_repeat(&self, value: String) -> bool {
        let _value = match value.as_str() {
            "None" => MediaPlaybackAutoRepeatMode::None,
            "List" => MediaPlaybackAutoRepeatMode::List,
            "Track" => MediaPlaybackAutoRepeatMode::Track,
            _ => MediaPlaybackAutoRepeatMode::None,
        };
        self.player.set_repeat(_value).await
    }

    #[napi]
    pub async fn get_repeat(&self) -> Option<String> {
        if let Some(repeat_mode) = self.player.get_repeat() {
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
        self.player.seek(offset_us).await
    }

    #[napi]
    pub async fn seek_percentage(&self, percentage: f64) -> bool {
        self.player.seek_percentage(percentage).await
    }

    #[napi]
    pub async fn set_position(&self, position_s: f64) -> bool {
        self.player.set_position(position_s).await
    }

    #[napi]
    pub async fn get_position(&self, wants_current_position: bool) -> Option<Position> {
        self.player.get_position(wants_current_position).await
    }
}
