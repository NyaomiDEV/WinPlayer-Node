use chrono::{DateTime, Utc};
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

use crate::owo::types::{ArtData, Capabilities, Metadata, Position, Status};

#[napi(object, js_name = "ArtData")]
pub struct JsArtData {
    pub data: Buffer,
    pub mimetype: String,
}

impl From<ArtData> for JsArtData {
    fn from(value: ArtData) -> Self {
        JsArtData {
            data: value.data.into(),
            mimetype: value.mimetype,
        }
    }
}

#[napi(object, js_name = "Metadata")]
pub struct JsMetadata {
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub album_artists: Option<Vec<String>>,
    pub artist: String,
    pub artists: Vec<String>,
    pub art_data: Option<JsArtData>,
    pub id: Option<String>,
    pub length: f64,
    pub title: String,
}

impl From<Metadata> for JsMetadata {
    fn from(value: Metadata) -> Self {
        JsMetadata {
            album: value.album,
            album_artist: value.album_artist,
            album_artists: value.album_artists,
            artist: value.artist,
            artists: value.artists,
            art_data: 'rt: {
                if let Some(art_data) = value.art_data {
                    break 'rt Some(JsArtData::from(art_data));
                };
                None
            },
            id: value.id,
            length: value.length,
            title: value.title,
        }
    }
}

#[napi(object, js_name = "Capabilities")]
pub struct JsCapabilities {
    pub can_control: bool,
    pub can_play_pause: bool,
    pub can_go_next: bool,
    pub can_go_previous: bool,
    pub can_seek: bool,
}

impl From<Capabilities> for JsCapabilities {
    fn from(value: Capabilities) -> Self {
        JsCapabilities {
            can_control: value.can_control,
            can_play_pause: value.can_play_pause,
            can_go_next: value.can_go_next,
            can_go_previous: value.can_go_previous,
            can_seek: value.can_seek,
        }
    }
}

#[napi(object, js_name = "Position")]
pub struct JsPosition {
    pub how_much: f64,
    pub when: DateTime<Utc>,
}

impl From<Position> for JsPosition {
    fn from(value: Position) -> Self {
        JsPosition {
            how_much: value.how_much,
            when: value.when,
        }
    }
}

#[napi(object, js_name = "Status")]
pub struct JsStatus {
    pub metadata: Option<JsMetadata>,
    pub capabilities: JsCapabilities,
    pub status: String,
    pub is_loop: String,
    pub shuffle: bool,
    pub volume: f64, // tanto sta a -1 lmao
    pub elapsed: Option<JsPosition>,
    pub app: Option<String>, // App User Model ID
}

impl From<Status> for JsStatus {
    fn from(value: Status) -> Self {
        JsStatus {
            metadata: 'rt: {
                if let Some(metadata) = value.metadata {
                    break 'rt Some(JsMetadata::from(metadata));
                };
                None
            },
            capabilities: JsCapabilities::from(value.capabilities),
            status: value.status,
            is_loop: value.is_loop,
            shuffle: value.shuffle,
            volume: value.volume,
            elapsed: 'rt: {
                if let Some(elapsed) = value.elapsed {
                    break 'rt Some(JsPosition::from(elapsed));
                };
                None
            },
            app: value.app,
        }
    }
}
