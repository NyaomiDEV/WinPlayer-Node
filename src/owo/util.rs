use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};

use windows::{
    core::HSTRING,
    ApplicationModel,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionPlaybackInfo,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
        GlobalSystemMediaTransportControlsSessionTimelineProperties,
    },
    Storage::Streams::{self, DataReader},
    System,
};

use crate::owo::types::{ArtData, Capabilities, Metadata, Position};

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

        let mut when: DateTime<Utc> = {
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
            let now = Utc::now();
            let time_from_last_update = now.timestamp_millis() - when.timestamp_millis();
            position += time_from_last_update as f64 / 1000f64;
            when = now;
        }

        return Some(Position {
            how_much: position,
            when,
        });
    }
    None
}

async fn get_session_player_name_for_user(aumid: &String) -> Option<String> {
    let user = {
        let user = System::User::FindAllAsync()
            .ok()?
            .await
            .ok()?
            .GetAt(0)
            .ok()?;
        Some(user)
    }?;

    let player_name =
        ApplicationModel::AppInfo::GetFromAppUserModelIdForUser(&user, &HSTRING::from(aumid))
            .ok()?
            .DisplayInfo()
            .ok()?
            .DisplayName()
            .ok()?;

    if player_name.to_string().eq(aumid) && player_name.to_string().ends_with(".exe") {
        return Some(
            player_name
                .to_string()
                .strip_suffix(".exe")
                .unwrap_or_default()
                .to_string(),
        );
    }

    Some(player_name.to_string())
}

async fn get_session_player_name_global(aumid: &String) -> Option<String> {
    let player_name = ApplicationModel::AppInfo::GetFromAppUserModelId(&HSTRING::from(aumid))
        .ok()?
        .DisplayInfo()
        .ok()?
        .DisplayName()
        .ok()?;

    if player_name.to_string().eq(aumid) && player_name.to_string().ends_with(".exe") {
        return Some(
            player_name
                .to_string()
                .strip_suffix(".exe")
                .unwrap_or_default()
                .to_string(),
        );
    }

    Some(player_name.to_string())
}

pub async fn get_session_player_name(aumid: &String) -> Option<String> {
    get_session_player_name_for_user(aumid)
        .await
        .or(get_session_player_name_global(aumid).await)
}

pub fn get_session_capabilities(
    session: &GlobalSystemMediaTransportControlsSession,
) -> Capabilities {
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

pub fn get_session_metadata(
    session: &GlobalSystemMediaTransportControlsSession,
) -> Option<Metadata> {
    let timeline_properties = session.GetTimelineProperties().unwrap();
    match session.TryGetMediaPropertiesAsync().unwrap().get() {
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

            let id = format!(
                "{}{}{}{}",
                metadata.album_artist.clone().unwrap_or(String::new()),
                metadata.artist,
                metadata.album.clone().unwrap_or(String::new()),
                metadata.title
            );
            if !id.is_empty() {
                let md5 = md5::compute(id);
                metadata.id = Some(format!("{:x}", md5).to_string());
            }

            if let Ok(thumbnail) = info.Thumbnail() {
                let stream = thumbnail.OpenReadAsync().unwrap().get().unwrap();

                if stream.CanRead().unwrap() && stream.Size().unwrap() > 0 {
                    let result_buffer = {
                        let buffer =
                            Streams::Buffer::Create(stream.Size().unwrap().try_into().unwrap())
                                .unwrap();

                        stream
                            .ReadAsync(
                                &buffer,
                                stream.Size().unwrap().try_into().unwrap(),
                                Streams::InputStreamOptions::None,
                            )
                            .unwrap()
                            .get()
                            .unwrap()
                    };

                    stream.FlushAsync().unwrap().get().unwrap();
                    stream.Close().unwrap();

                    let data_reader = DataReader::FromBuffer(&result_buffer).unwrap();
                    let mut data = Vec::with_capacity(result_buffer.Length().unwrap() as usize);
                    data_reader.ReadBytes(&mut data).unwrap();

                    dbg!(&data);

                    metadata.art_data = Some(ArtData {
                        data,
                        mimetype: vec![stream.ContentType().unwrap().to_string()],
                    });
                }
            }

            Some(metadata)
        }
        Err(_) => None,
    }
}
