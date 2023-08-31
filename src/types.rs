struct ArtData {
    data: Vec<u8>,
    mimetype: Vec<String>,
}

struct Metadata {
    id: String,
    title: String,
    artist: String,
    artists: Vec<String>,
    album: String,
    album_artist: String,
    album_artists: Vec<String>,
    art_data: ArtData,
    length: f32,
}

struct Capabilities {
    can_control: bool,
    can_play_pause: bool,
    can_go_next: bool,
    can_go_previous: bool,
    can_seek: bool,
}

struct Update {
    metadata: Option<Metadata>,
    capabilities: Capabilities,
    status: String,
    is_loop: String,
    shuffle: bool,
    volume: f32, // tanto sta a -1 lmao
    elapsed: f32,
    app: String,      // App User Model ID
    app_name: String, // Nome per hoomans
}
