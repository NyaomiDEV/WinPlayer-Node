use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};

use windows::{
    core::{Error, HSTRING},
    ApplicationModel,
    Foundation::TypedEventHandler,
    Graphics::Imaging,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Media::MediaPlaybackAutoRepeatMode,
    Security::Cryptography::{BinaryStringEncoding, Core, CryptographicBuffer},
    Storage::Streams::{self, DataReader},
    System,
};

use crate::types::{ArtData, Capabilities, Metadata, Position, Update};

// I don't want to deal with libraries
fn shitty_windows_epoch_to_actually_usable_unix_timestamp(shitty_time: i64) -> i64 {
    // 64-bit value representing the number of 100-nanosecond intervals since January 1, 1601 (UTC)
    const TICKS_PER_MILLISECOND: i64 = 10000;
    const UNIX_TIMESTAMP_DIFFERENCE: i64 = 0x019DB1DED53E8000;
    (shitty_time - UNIX_TIMESTAMP_DIFFERENCE) / TICKS_PER_MILLISECOND
}

async fn get_session_player_name_for_user(
    session: &GlobalSystemMediaTransportControlsSession,
) -> Result<String, Error> {
    let mut player_name = session.SourceAppUserModelId()?;
    let user = System::User::FindAllAsync()?.await?.GetAt(0)?;

    player_name = ApplicationModel::AppInfo::GetFromAppUserModelIdForUser(&user, &player_name)?
        .DisplayInfo()?
        .DisplayName()?;

    if session.SourceAppUserModelId().unwrap() == player_name
        && player_name.to_string().ends_with(".exe")
    {
        player_name = HSTRING::from(
            player_name
                .to_string()
                .strip_suffix(".exe")
                .unwrap_or_default(),
        );
    }

    Ok(player_name.to_string())
}

async fn get_session_player_name_global(
    session: &GlobalSystemMediaTransportControlsSession,
) -> Result<String, Error> {
    let mut player_name = session.SourceAppUserModelId()?;

    player_name = ApplicationModel::AppInfo::GetFromAppUserModelId(&player_name)?
        .DisplayInfo()?
        .DisplayName()?;

    if session.SourceAppUserModelId().unwrap() == player_name
        && player_name.to_string().ends_with(".exe")
    {
        player_name = HSTRING::from(
            player_name
                .to_string()
                .strip_suffix(".exe")
                .unwrap_or_default(),
        );
    }

    Ok(player_name.to_string())
}

async fn get_session_player_name(
    session: &GlobalSystemMediaTransportControlsSession,
) -> Result<String, Error> {
    match get_session_player_name_for_user(&session).await {
        Ok(r) => Ok(r),
        Err(_) => match get_session_player_name_global(&session).await {
            Ok(r) => Ok(r),
            Err(e) => Err(e),
        },
    }
}

fn get_session_capabilities(session: &GlobalSystemMediaTransportControlsSession) -> Capabilities {
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
    session: &GlobalSystemMediaTransportControlsSession,
) -> Option<Metadata> {
    let timeline_properties = session.GetTimelineProperties().unwrap();
    match session.TryGetMediaPropertiesAsync().unwrap().await {
        Ok(info) => {
            let mut metadata = Metadata {
                album: info.AlbumTitle().ok().map(|x| x.to_string()),
                album_artist: info.AlbumArtist().ok().map(|x| x.to_string()),
                album_artists: (|| {
                    let _artist = info.AlbumArtist().ok().map(|x| x.to_string());
                    _artist.as_ref()?;
                    Some(vec![_artist.unwrap()])
                })(),
                artist: info.Artist().unwrap_or_default().to_string(),
                artists: vec![info.Artist().unwrap_or_default().to_string()],
                art_data: None,
                id: None, // md5 di String(album_artist + artist + album + title)
                length: (timeline_properties.EndTime().unwrap_or_default().Duration
                    - timeline_properties.StartTime().unwrap_or_default().Duration)
                    as f64
                    / 1000f64,
                title: info.Title().unwrap_or_default().to_string(),
            };

            let id = HSTRING::from(format!(
                "{}{}{}{}",
                metadata.album_artist.clone().unwrap_or(String::new()),
                metadata.artist,
                metadata.album.clone().unwrap_or(String::new()),
                metadata.title
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
                metadata.id = Some(
                    CryptographicBuffer::EncodeToHexString(&md5.HashData(&id_buf).unwrap())
                        .unwrap()
                        .to_string(),
                );
            }

            let thumbnail = info.Thumbnail();
            if thumbnail.is_ok() {
                let stream = thumbnail.unwrap().OpenReadAsync().unwrap().await.unwrap();
                if stream.CanRead().unwrap() && stream.Size().unwrap() > 0 {
                    let decoder = Imaging::BitmapDecoder::CreateAsync(&stream)
                        .unwrap()
                        .await
                        .unwrap();

                    let pngstream = Streams::InMemoryRandomAccessStream::new().unwrap();
                    let encoder = Imaging::BitmapEncoder::CreateAsync(
                        Imaging::BitmapEncoder::PngEncoderId().unwrap(),
                        &pngstream,
                    )
                    .unwrap()
                    .await
                    .unwrap();

                    let software_bitmap = decoder.GetSoftwareBitmapAsync().unwrap().await.unwrap();
                    encoder.SetSoftwareBitmap(&software_bitmap).unwrap();

                    encoder.FlushAsync().unwrap().await.unwrap();

                    let buffer =
                        Streams::Buffer::Create(pngstream.Size().unwrap().try_into().unwrap())
                            .unwrap();
                    let result_buffer = pngstream
                        .ReadAsync(
                            &buffer,
                            pngstream.Size().unwrap().try_into().unwrap(),
                            Streams::InputStreamOptions::None,
                        )
                        .unwrap()
                        .await
                        .unwrap();
                    pngstream.FlushAsync().unwrap().await.unwrap();
                    pngstream.Close().unwrap();

                    let data_reader = DataReader::FromBuffer(&result_buffer).unwrap();
                    let mut data = Vec::with_capacity(result_buffer.Length().unwrap() as usize);
                    data_reader.ReadBytes(&mut data).unwrap();

                    metadata.art_data = Some(ArtData {
                        data,
                        mimetype: vec![String::from("image/png")],
                    });
                }
            }

            Some(metadata)
        }
        Err(_) => None,
    }
}

async fn get_session_status(session: GlobalSystemMediaTransportControlsSession) -> Update {
    let playback_info = session.GetPlaybackInfo().ok();
    let timeline_properties = session.GetTimelineProperties().ok();

    Update {
        metadata: get_session_metadata(&session).await,
        capabilities: get_session_capabilities(&session),
        status: 'rt: {
            if playback_info.is_none() {
                break 'rt String::from("Stopped");
            }
            let _status = playback_info.as_ref().unwrap().PlaybackStatus().ok();
            match _status.unwrap() {
                GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => {
                    String::from("Playing")
                }
                GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => {
                    String::from("Paused")
                }
                _ => String::from("Stopped"),
            }
        },
        is_loop: 'rt: {
            if playback_info.is_none() {
                break 'rt String::from("None");
            }
            let _mode = playback_info.as_ref().unwrap().AutoRepeatMode().ok();
            if _mode.is_none() {
                break 'rt String::from("None");
            }
            match _mode
                .unwrap()
                .Value()
                .unwrap_or(MediaPlaybackAutoRepeatMode::None)
            {
                MediaPlaybackAutoRepeatMode::List => String::from("List"),
                MediaPlaybackAutoRepeatMode::Track => String::from("Track"),
                _ => String::from("None"),
            }
        },
        shuffle: 'rt: {
            if playback_info.is_none() {
                break 'rt false;
            }
            let _shuffle = playback_info.unwrap().IsShuffleActive().ok();
            if _shuffle.is_none() {
                break 'rt false;
            }
            _shuffle.unwrap().Value().unwrap_or(false)
        },
        volume: -1f64,
        elapsed: 'rt: {
            if timeline_properties.is_none() {
                break 'rt Position {
                    how_much: 0f64,
                    when: Utc.timestamp_millis_opt(0).unwrap(),
                };
            }
            let _props = timeline_properties.unwrap();

            if _props.Position().ok().is_none() || _props.StartTime().ok().is_none() {
                break 'rt Position {
                    how_much: 0f64,
                    when: Utc.timestamp_millis_opt(0).unwrap(),
                };
            }

            let _position: Duration = _props.Position().unwrap().into();
            let _start_time: Duration = _props.StartTime().unwrap().into();
            let _when: DateTime<Utc> = Utc
                .timestamp_millis_opt(shitty_windows_epoch_to_actually_usable_unix_timestamp(
                    _props.LastUpdatedTime().unwrap().UniversalTime,
                ))
                .unwrap();

            Position {
                how_much: (_position.as_secs_f32() - _start_time.as_secs_f32()) as f64,
                when: _when,
            }
        },
        app: 'rt: {
            let aumid = session.SourceAppUserModelId().ok();
            if aumid.is_none() {
                break 'rt None::<String>;
            }
            Some(aumid.unwrap().to_string())
        },
        app_name: 'rt: {
            let app_name = get_session_player_name(&session).await.ok();
            if app_name.is_none() {
                break 'rt None::<String>;
            }
            Some(app_name.unwrap())
        },
    }
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

    pub async fn get_active_session_status(&self) -> Option<Update> {
        match self.get_session() {
            None => None,
            Some(session) => Some(get_session_status(session).await),
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

    pub async fn get_position(&self) -> Result<Position, &str> {
        match self.get_session() {
            None => Err("No active session"),
            Some(session) => match session.GetTimelineProperties() {
                Ok(timeline_properties) => {
                    if timeline_properties.EndTime().unwrap_or_default().Duration == 0 {
                        return Ok(Position {
                            how_much: 0f64,
                            when: Utc.timestamp_millis_opt(0).unwrap(),
                        });
                    }

                    let _position: Duration =
                        timeline_properties.Position().unwrap_or_default().into();
                    let mut position: f64 = _position.as_secs_f64();
                    let playback_status =
                        session.GetPlaybackInfo().unwrap().PlaybackStatus().unwrap();

                    let _when: DateTime<Utc> = Utc
                        .timestamp_millis_opt(
                            shitty_windows_epoch_to_actually_usable_unix_timestamp(
                                timeline_properties.LastUpdatedTime().unwrap().UniversalTime,
                            ),
                        )
                        .unwrap();

                    if playback_status
                        == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
                    {
                        let time_from_last_update = chrono::offset::Utc::now().timestamp_millis()
                            - _when.timestamp_millis();
                        position += time_from_last_update as f64 / 1000f64;
                    }

                    Ok(Position {
                        how_much: position,
                        when: _when,
                    })
                }
                Err(_) => Err("Could not get timeline properties"),
            },
        }
    }
}
