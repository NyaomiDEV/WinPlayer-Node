use chrono::{DateTime, Utc};
use napi_derive::napi;

pub type CallbackFn = dyn Fn(String) + Send + Sync;

#[napi(object)]
pub struct ArtData {
    pub data: Vec<u8>,
    pub mimetype: Vec<String>,
}

#[napi(object)]
pub struct Metadata {
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub album_artists: Option<Vec<String>>,
    pub artist: String,
    pub artists: Vec<String>,
    pub art_data: Option<ArtData>,
    pub id: Option<String>,
    pub length: f64,
    pub title: String,
}

#[napi(object)]
pub struct Capabilities {
    pub can_control: bool,
    pub can_play_pause: bool,
    pub can_go_next: bool,
    pub can_go_previous: bool,
    pub can_seek: bool,
}

#[napi(object)]
pub struct Position {
    pub how_much: f64,
    pub when: DateTime<Utc>,
}

#[napi(object)]
pub struct Status {
    pub metadata: Option<Metadata>,
    pub capabilities: Capabilities,
    pub status: String,
    pub is_loop: String,
    pub shuffle: bool,
    pub volume: f64, // tanto sta a -1 lmao
    pub elapsed: Option<Position>,
    pub app: Option<String>, // App User Model ID
}
