pub struct ArtData {
    pub data: Vec<u8>,
    pub mimetype: Vec<String>,
}

pub struct Metadata {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub artists: Vec<String>,
    pub album: String,
    pub album_artist: String,
    pub album_artists: Vec<String>,
    pub art_data: ArtData,
    pub length: f64,
}

pub struct Capabilities {
    pub can_control: bool,
    pub can_play_pause: bool,
    pub can_go_next: bool,
    pub can_go_previous: bool,
    pub can_seek: bool,
}

pub struct Update {
    pub metadata: Option<Metadata>,
    pub capabilities: Capabilities,
    pub status: String,
    pub is_loop: String,
    pub shuffle: bool,
    pub volume: f64,      // tanto sta a -1 lmao
    pub elapsed: f64,
    pub app: String,      // App User Model ID
    pub app_name: String, // Nome per hoomans
}
