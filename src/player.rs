use std::sync::{Arc, Mutex};

use chrono;

use windows::{
    core::HSTRING,
    ApplicationModel, Foundation,
    Foundation::{Collections, TypedEventHandler},
    Graphics::Imaging,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Media::MediaPlaybackAutoRepeatMode,
    Security::Cryptography::{BinaryStringEncoding, Core, CryptographicBuffer},
    Storage::Streams,
    System,
};

use crate::types::{Capabilities, Metadata, Update};

fn get_session_capabilities(session: GlobalSystemMediaTransportControlsSession) -> Capabilities {
    let controls = session.GetPlaybackInfo().unwrap().Controls().unwrap();

    let mut capabilities = Capabilities {
        can_play_pause: controls.IsPlayEnabled().unwrap_or(false)
            || controls.IsPauseEnabled().unwrap_or(false),
        can_go_next: controls.IsNextEnabled().unwrap_or(false),
        can_go_previous: controls.IsPreviousEnabled().unwrap_or(false),
        can_seek: controls.IsPlaybackPositionEnabled().unwrap_or(false)
            && session
                .GetTimelineProperties()
                .unwrap()
                .EndTime()
                .unwrap_or_default()
                .Duration
                != 0,
        can_control: false,
    };
    capabilities.can_control = capabilities.can_play_pause
        || capabilities.can_go_next
        || capabilities.can_go_previous
        || capabilities.can_seek;

    capabilities
}

async fn get_session_metadata(
    session: GlobalSystemMediaTransportControlsSession,
) -> Option<Metadata> {
    let timeline_properties = session.GetTimelineProperties().unwrap();
    match session.TryGetMediaPropertiesAsync().unwrap().await {
        Ok(info) => {
            let mut metadata = Metadata {
                album: info.AlbumTitle().unwrap_or_default().to_string(),
                album_artist: info.AlbumArtist().unwrap_or_default().to_string(),
                album_artists: vec![info.AlbumArtist().unwrap_or_default().to_string()],
                artist: info.Artist().unwrap_or_default().to_string(),
                artists: vec![info.Artist().unwrap_or_default().to_string()],
                art_data: None,
                id: String::new(), // md5 di String(album_artist + artist + album + title)
                length: (timeline_properties.EndTime().unwrap_or_default().Duration
                    - timeline_properties.StartTime().unwrap_or_default().Duration)
                    as f64
                    / 1000f64,
                title: info.Title().unwrap_or_default().to_string(),
            };

            let id = HSTRING::from(format!(
                "{}{}{}{}",
                metadata.album_artist, metadata.artist, metadata.album, metadata.title
            ));
            // TODO: Fare MD5 hashing con qualcosa Rust standard... ma avevo questo dall'altro lato
            if !id.is_empty() {
                let md5 = Core::HashAlgorithmProvider::OpenAlgorithm(
                    &Core::HashAlgorithmNames::Md5().unwrap(),
                )
                .unwrap();
                let id_buf =
                    CryptographicBuffer::ConvertStringToBinary(&id, BinaryStringEncoding::Utf8)
                        .unwrap();
                metadata.id =
                    CryptographicBuffer::EncodeToHexString(&md5.HashData(&id_buf).unwrap())
                        .unwrap()
                        .to_string();
            }

            /* C++ da portare
            auto thumbnail = info.Thumbnail();
            if (thumbnail){
            auto stream = co_await thumbnail.OpenReadAsync();
            if (stream.CanRead() && stream.Size() > 0){
                winrt::Windows::Graphics::Imaging::BitmapDecoder decoder = co_await winrt::Windows::Graphics::Imaging::BitmapDecoder::CreateAsync(stream);
                auto softwareBitmap = co_await decoder.GetSoftwareBitmapAsync();

                auto pngstream = winrt::Windows::Storage::Streams::InMemoryRandomAccessStream::InMemoryRandomAccessStream();
                auto encoder = co_await winrt::Windows::Graphics::Imaging::BitmapEncoder::CreateAsync(
                    winrt::Windows::Graphics::Imaging::BitmapEncoder::PngEncoderId(),
                    pngstream
                );
                encoder.SetSoftwareBitmap(softwareBitmap);
                co_await encoder.FlushAsync();

                 winrt::Windows::Storage::Streams::IBuffer buffer = winrt::Windows::Storage::Streams::Buffer(pngstream.Size());
                buffer = co_await pngstream.ReadAsync(buffer, pngstream.Size(), winrt::Windows::Storage::Streams::InputStreamOptions::None);
                co_await pngstream.FlushAsync();
                pngstream.Close();

                auto data = buffer.data();

                metadata.artData.data = std::vector<uint8_t>(&data[0], &data[buffer.Length() - 1]);
                metadata.artData.type.push_back("image/png");
            }
            */
            Some(metadata)
        }
        Err(_) => None,
    }
}

fn get_session_status(session: GlobalSystemMediaTransportControlsSession) -> Result<Update, &str> {
    Err("Da implementare")
}

struct Player {
    session_manager: GlobalSystemMediaTransportControlsSessionManager,
    active_player: Option<String>,
}

impl Player {
    pub async fn new() -> Self {
        let session_manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .expect("The session manager is kil")
            .await
            .expect("The session manager is kil 2");

        Player {
            session_manager,
            active_player: None,
        }
    }

    pub fn run(self) {
        // Possiamo autostartarla dal costruttore o integrarla a esso?
        // Passando self così non rischiamo di perdercelo dopo questa call?
        let rc_self = Arc::new(Mutex::new(self));

        let handler = TypedEventHandler::new({
            let s = rc_self.clone();
            move |_, _| {
                Ok({
                    let mut binding = s.lock();
                    let s = binding.as_mut().unwrap();
                    let preferred = s.active_player.clone();
                    s.update_active_session(preferred);
                })
            }
        });

        rc_self
            .lock()
            .unwrap()
            .session_manager
            .SessionsChanged(&handler);

        let preferred = rc_self.lock().unwrap().active_player.clone();
        rc_self.lock().unwrap().update_active_session(preferred);
    }

    async fn get_session_player_name(
        session: GlobalSystemMediaTransportControlsSession,
    ) -> Result<String, &str> {
        // Tecnicamente va dato un Err a ogni unwrap fallito. Non so che pattern usare.
        let mut player_name = session.SourceAppUserModelId().unwrap();
        let user = System::User::FindAllAsync()
            .unwrap()
            .await
            .unwrap()
            .GetAt(0)
            .unwrap();

        player_name = ApplicationModel::AppInfo::GetFromAppUserModelIdForUser(&user, &player_name)
            .unwrap()
            .DisplayInfo()
            .unwrap()
            .DisplayName()
            .unwrap();

        if session.SourceAppUserModelId().unwrap() == player_name
            && player_name.to_string().ends_with(".exe")
        {
            player_name = HSTRING::from(player_name.to_string().strip_suffix(".exe").unwrap());
        }

        Ok(player_name.to_string()) // ok come torniamo Err a ogni expect?
    }

    fn get_session(&self) -> Option<GlobalSystemMediaTransportControlsSession> {
        match self.session_manager.GetSessions() {
            Ok(ses) => {
                let active_player = self.active_player.clone().unwrap_or_default();
                for session in ses {
                    if session.SourceAppUserModelId().unwrap().to_string() == active_player {
                        return Some(session);
                    }
                }
                None
            }
            Err(e) => None,
        }
    }

    fn update_active_session(&mut self, preferred: Option<String>) {
        self.active_player = None;

        let sessions = self.session_manager.GetSessions().expect("No sessions?");
        let preferred = preferred.unwrap_or_default();
        for session in sessions {
            if (session.SourceAppUserModelId().unwrap().to_string() == preferred)
                || session.GetPlaybackInfo().unwrap().PlaybackStatus().unwrap()
                    == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
            {
                self.active_player = Some(session.SourceAppUserModelId().unwrap().to_string());
                break;
            }
        }
    }

    pub fn get_active_session_status(&self) -> Result<Update, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => get_session_status(session),
        }
    }

    pub async fn play(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => match session.TryPlayAsync() {
                Ok(result) => Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    pub async fn pause(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => match session.TryPauseAsync() {
                Ok(result) => Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    pub async fn play_pause(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => match session.TryTogglePlayPauseAsync() {
                Ok(result) => Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    pub async fn stop(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => match session.TryStopAsync() {
                Ok(result) => Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    pub async fn next(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => match session.TrySkipNextAsync() {
                Ok(result) => Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    pub async fn previous(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => match session.TrySkipPreviousAsync() {
                Ok(result) => Ok(result.await.unwrap_or(false)),
                Err(_) => Err("Error while trying to perform the command"),
            },
        }
    }

    pub async fn shuffle(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => match session.GetPlaybackInfo() {
                Ok(playback_info) => match playback_info.IsShuffleActive() {
                    Ok(shuffle_active) => {
                        match session
                            .TryChangeShuffleActiveAsync(shuffle_active.Value().unwrap_or(false))
                        {
                            Ok(async_operation) => match async_operation.await {
                                Ok(result) => Ok(result),
                                Err(_) => Ok(false),
                            },
                            Err(_) => Err("Could not change shuffle mode"),
                        }
                    }
                    Err(_) => Ok(false),
                },
                Err(_) => Err("Could not get playback info"),
            },
        }
    }

    pub async fn repeat(&self) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => match session.GetPlaybackInfo() {
                Ok(playback_info) => match playback_info.AutoRepeatMode() {
                    Ok(repeat_mode) => {
                        let new_repeat_mode = match repeat_mode.Value() {
                            Err(_) => MediaPlaybackAutoRepeatMode::None,
                            Ok(rp) => match rp {
                                MediaPlaybackAutoRepeatMode::None => {
                                    MediaPlaybackAutoRepeatMode::List
                                }
                                MediaPlaybackAutoRepeatMode::List => {
                                    MediaPlaybackAutoRepeatMode::Track
                                }
                                MediaPlaybackAutoRepeatMode::Track => {
                                    MediaPlaybackAutoRepeatMode::None
                                }
                                _ => MediaPlaybackAutoRepeatMode::None,
                            },
                        };
                        match session.TryChangeAutoRepeatModeAsync(new_repeat_mode) {
                            Ok(async_operation) => match async_operation.await {
                                Ok(result) => Ok(result),
                                Err(_) => Ok(false),
                            },
                            Err(_) => Err("Could not change repeat mode"),
                        }
                    }
                    Err(_) => Ok(false),
                },
                Err(_) => Err("Could not get playback info"),
            },
        }
    }

    pub async fn seek(&self, offset_us: i64) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => match session.GetTimelineProperties() {
                Ok(timeline_properties) => {
                    let position = timeline_properties.Position().unwrap_or_default();
                    self.set_position((position.Duration + offset_us) as f64 / 1000f64)
                        .await
                }
                Err(_) => Err("Could not get timeline properties"),
            },
        }
    }

    pub async fn seek_percentage(&self, percentage: f64) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => match session.GetTimelineProperties() {
                Ok(timeline_properties) => {
                    let start_time = timeline_properties.StartTime().unwrap_or_default();
                    let end_time = timeline_properties.EndTime().unwrap_or_default();
                    let length = (end_time.Duration - start_time.Duration) as f64 / 1000.0;
                    self.set_position(length * percentage).await
                }
                Err(_) => Err("Could not get timeline properties"),
            },
        }
    }

    pub async fn set_position(&self, position_s: f64) -> Result<bool, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => {
                match session.TryChangePlaybackPositionAsync((position_s * 1000.0) as i64) {
                    // probabilmente non worka e la pos sara' wonky
                    Ok(result) => Ok(result.await.unwrap_or(false)),
                    Err(_) => Err("Error while trying to perform the command"),
                }
            }
        }
    }

    pub async fn get_position(&self) -> Result<f64, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => match session.GetTimelineProperties() {
                Ok(timeline_properties) => {
                    if timeline_properties.EndTime().unwrap_or_default().Duration == 0 {
                        return Ok(0f64);
                    }

                    let mut position = timeline_properties.Position().unwrap_or_default().Duration;
                    let playback_status =
                        session.GetPlaybackInfo().unwrap().PlaybackStatus().unwrap();

                    if playback_status
                        == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
                    {
                        let time_from_last_update = chrono::offset::Utc::now().timestamp_millis()
                            - timeline_properties
                                .LastUpdatedTime()
                                .unwrap_or(Foundation::DateTime { UniversalTime: 0 })
                                .UniversalTime;
                        position += time_from_last_update;
                    }

                    Ok(
                        (position - timeline_properties.StartTime().unwrap().Duration) as f64
                            / 1000f64,
                    )
                }
                Err(_) => Err("Could not get timeline properties"),
            },
        }
    }
}
