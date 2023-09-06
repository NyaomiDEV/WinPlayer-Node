use napi::{threadsafe_function::{ThreadsafeFunction, ErrorStrategy, ThreadsafeFunctionCallMode}, JsFunction};
use napi_derive::napi;
use windows::Media::MediaPlaybackAutoRepeatMode;

use crate::owo::{player::Player, types::{Status, Position}};

#[napi(js_name = "Player")]
pub struct JsPlayer {
    player: Player,
	event_callback_tsfn: Option<ThreadsafeFunction<Vec<String>, ErrorStrategy::Fatal>>
}

#[napi]
impl JsPlayer {
	#[napi]
    pub fn wrap_player(player: Player) -> Self {
        JsPlayer { player, event_callback_tsfn: None }
    }

	#[napi]
    pub fn set_event_callback(&mut self, callback: JsFunction) {
        self.event_callback_tsfn = Some(callback.create_threadsafe_function(0, |ctx| {
            Ok(ctx.value)
        }).unwrap());

        self.player.set_event_callback(Box::new(|event| {
			if let Some(tsfn) = self.event_callback_tsfn {
            	tsfn.call(vec![event], ThreadsafeFunctionCallMode::NonBlocking);
			}
        }));
    }

	#[napi]
    pub fn unset_event_callback(&mut self) {
		self.player.unset_event_callback()
    }

	#[napi]
    pub async unsafe fn get_session_status(&mut self) -> Status {
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
    pub async fn shuffle(&self) -> bool {
        self.player.shuffle().await
    }

	#[napi]
    pub async fn set_repeat(&self, value: String) -> bool {
		let _value = match value.as_str() {
			"None" => MediaPlaybackAutoRepeatMode::None,
			"List" => MediaPlaybackAutoRepeatMode::List,
			"Track" => MediaPlaybackAutoRepeatMode::Track,
			_ => MediaPlaybackAutoRepeatMode::None
		};
        self.player.set_repeat(_value).await
    }

	#[napi]
    pub async fn repeat(&self) -> bool {
        self.player.repeat().await
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
