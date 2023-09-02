use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};

use windows::{
    core::{Error, HSTRING},
    ApplicationModel,
    Graphics::Imaging,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionPlaybackInfo,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
        GlobalSystemMediaTransportControlsSessionTimelineProperties,
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

pub fn compute_position(
    timeline_properties: Option<&GlobalSystemMediaTransportControlsSessionTimelineProperties>,
    playback_info: Option<&GlobalSystemMediaTransportControlsSessionPlaybackInfo>,
    account_for_time_skew: bool,
) -> Option<Position> {
    if let Some(timeline_properties) = timeline_properties {
        let playback_status = 'rt: {
            if let Some(playback_info) = playback_info {
                if let Ok(playback_status) = playback_info.PlaybackStatus() {
                    break 'rt playback_status;
                }
            }
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped
        };

        let when: DateTime<Utc> = {
            let mut timestamp = 0;
            if let Ok(last_updated_time) = timeline_properties.LastUpdatedTime() {
                timestamp = shitty_windows_epoch_to_actually_usable_unix_timestamp(
                    last_updated_time.UniversalTime,
                );
            }
            Utc.timestamp_millis_opt(timestamp).unwrap()
        };

        let end_time: f64 = 'rt2: {
            if let Ok(_end_time) = timeline_properties.EndTime() {
                let _duration: Duration = _end_time.into();
                break 'rt2 _duration.as_secs_f64();
            }
            0f64
        };

        let mut position: f64 = 'rt2: {
            if let Ok(_position) = timeline_properties.Position() {
                if let Ok(_start_time) = timeline_properties.StartTime() {
                    let _duration: Duration = _position.into();
                    let _start_duration: Duration = _start_time.into();
                    break 'rt2 _duration.as_secs_f64() - _start_duration.as_secs_f64();
                }
            }
            0f64
        };

        if end_time == 0f64 {
            return None;
        }

        if account_for_time_skew
            && playback_status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
        {
            let time_from_last_update =
                chrono::offset::Utc::now().timestamp_millis() - when.timestamp_millis();
            position += time_from_last_update as f64 / 1000f64;
        }

        return Some(Position {
            how_much: position,
            when,
        });
    }
    None
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

pub async fn get_session_player_name(
    session: &GlobalSystemMediaTransportControlsSession,
) -> Result<String, Error> {
    match get_session_player_name_for_user(session).await {
        Ok(r) => Ok(r),
        Err(_) => match get_session_player_name_global(session).await {
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

pub async fn get_session_status(session: GlobalSystemMediaTransportControlsSession) -> Update {
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
            let _shuffle = playback_info.as_ref().unwrap().IsShuffleActive().ok();
            if _shuffle.is_none() {
                break 'rt false;
            }
            _shuffle.unwrap().Value().unwrap_or(false)
        },
        volume: -1f64,
        elapsed: compute_position(timeline_properties.as_ref(), playback_info.as_ref(), false),
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
