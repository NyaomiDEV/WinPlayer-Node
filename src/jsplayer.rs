use std::sync::Arc;

use napi::bindgen_prelude::External;
use napi_derive::napi;
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};
use windows::Media::{MediaPlaybackAutoRepeatMode, Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus};

use crate::owo::{
    player::{Player, PlayerEvent},
    types::{Position, Status},
};

#[napi(js_name = "Player")]
pub struct JsPlayer {
    internal: External<JsPlayerInternal>,
}

pub struct JsPlayerInternal {
    player: Arc<Mutex<Player>>,
    rx: UnboundedReceiver<PlayerEvent>,
}

#[napi]
impl JsPlayer {
    #[napi(constructor)]
    pub fn new(internal: External<JsPlayerInternal>) -> Self {
        JsPlayer { internal }
    }

    pub async fn wrap_player(player: Arc<Mutex<Player>>) -> Self {
        let rx = player.lock().await.set_events();
        let internal = JsPlayerInternal { player, rx };
        JsPlayer::new(External::new(internal))
    }

    #[napi]
    pub async unsafe fn poll_next_event(&mut self) -> String {
        match self.internal.rx.recv().await.unwrap() {
            PlayerEvent::PlaybackInfoChanged => String::from("PlaybackInfoChanged"),
            PlayerEvent::MediaPropertiesChanged => String::from("MediaPropertiesChanged"),
            PlayerEvent::TimelinePropertiesChanged => String::from("TimelinePropertiesChanged"),
        }
    }

    #[napi]
    pub async fn get_status(&self) -> Status {
        self.internal.player.lock().await.get_status().await
    }

    #[napi]
    pub async fn get_aumid(&self) -> String {
        self.internal.player.lock().await.get_aumid()
    }

    #[napi]
    pub async fn play(&self) -> bool {
        self.internal.player.lock().await.play().await
    }

    #[napi]
    pub async fn pause(&self) -> bool {
        self.internal.player.lock().await.pause().await
    }

    #[napi]
    pub async fn play_pause(&self) -> bool {
        self.internal.player.lock().await.play_pause().await
    }

    #[napi]
    pub async fn stop(&self) -> bool {
        self.internal.player.lock().await.stop().await
    }

    #[napi]
    pub async fn get_playback_status(&self) -> String {
        match self.internal.player.lock().await.get_playback_status() {
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => String::from("Playing"),
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => String::from("Paused"),
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => String::from("Stopped"),
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Changing => String::from("Changing"),
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed => String::from("Closed"),
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Opened => String::from("Opened"),
            _ => String::from("Unknown")
        }
    }

    #[napi]
    pub async fn next(&self) -> bool {
        self.internal.player.lock().await.next().await
    }

    #[napi]
    pub async fn previous(&self) -> bool {
        self.internal.player.lock().await.previous().await
    }

    #[napi]
    pub async fn set_shuffle(&self, value: bool) -> bool {
        self.internal.player.lock().await.set_shuffle(value).await
    }

    #[napi]
    pub async fn get_shuffle(&self) -> bool {
        self.internal.player.lock().await.get_shuffle()
    }

    #[napi]
    pub async fn set_repeat(&self, value: String) -> bool {
        let _value = match value.as_str() {
            "None" => MediaPlaybackAutoRepeatMode::None,
            "List" => MediaPlaybackAutoRepeatMode::List,
            "Track" => MediaPlaybackAutoRepeatMode::Track,
            _ => MediaPlaybackAutoRepeatMode::None,
        };
        self.internal.player.lock().await.set_repeat(_value).await
    }

    #[napi]
    pub async fn get_repeat(&self) -> Option<String> {
        if let Some(repeat_mode) = self.internal.player.lock().await.get_repeat() {
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
    pub async fn seek(&self, offset_s: f64) -> bool {
        self.internal.player.lock().await.seek(offset_s).await
    }

    #[napi]
    pub async fn seek_percentage(&self, percentage: f64) -> bool {
        self.internal
            .player
            .lock()
            .await
            .seek_percentage(percentage)
            .await
    }

    #[napi]
    pub async fn set_position(&self, position_s: f64) -> bool {
        self.internal
            .player
            .lock()
            .await
            .set_position(position_s)
            .await
    }

    #[napi]
    pub async fn get_position(&self, wants_current_position: bool) -> Option<Position> {
        self.internal
            .player
            .lock()
            .await
            .get_position(wants_current_position)
            .await
    }
}
