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
    if let Ok(playback_info) = session.GetPlaybackInfo() {
        if let Ok(controls) = playback_info.Controls() {
            let mut capabilities = Capabilities {
                can_play_pause: controls.IsPlayEnabled().unwrap_or(false)
                    || controls.IsPauseEnabled().unwrap_or(false),
                can_go_next: controls.IsNextEnabled().unwrap_or(false),
                can_go_previous: controls.IsPreviousEnabled().unwrap_or(false),
                can_seek: {
                    let is_pp_enabled = controls.IsPlaybackPositionEnabled().unwrap_or(false);
                    let is_endtime = 'rt: {
                        if let Ok(p) = session.GetTimelineProperties() {
                            break 'rt p.EndTime().unwrap_or_default().Duration != 0;
                        }
                        false
                    };
                    is_pp_enabled && is_endtime
                },
                can_control: false,
            };
            capabilities.can_control = capabilities.can_play_pause
                || capabilities.can_go_next
                || capabilities.can_go_previous
                || capabilities.can_seek;

            return capabilities;
        }
    }
    Capabilities {
        can_control: false,
        can_play_pause: false,
        can_go_next: false,
        can_go_previous: false,
        can_seek: false,
    }
}

pub fn get_session_metadata(
    session: &GlobalSystemMediaTransportControlsSession,
) -> Option<Metadata> {
    if let Ok(timeline_properties) = session.GetTimelineProperties() {
        if let Ok(media_properties) = session.TryGetMediaPropertiesAsync() {
            if let Ok(info) = media_properties.get() {

                    let mut metadata = Metadata {
                        album: info.AlbumTitle().ok().map(|x| x.to_string()),
                        album_artist: info.AlbumArtist().ok().map(|x| x.to_string()),
                        album_artists: 'rt: {
                            if let Ok(artist) = info.AlbumArtist() {
                                break 'rt Some(vec![artist.to_string()]);
                            }
                            None
                        },
                        artist: info.Artist().unwrap_or_default().to_string(),
                        artists: vec![info.Artist().unwrap_or_default().to_string()],
                        art_data: None,
                        id: None,
                        length: {
                            let start_time = 'rt: {
                                if let Ok(_start) = timeline_properties.StartTime() {
                                    let _duration: Duration = _start.into();
                                    break 'rt _duration.as_secs_f64();
                                }
                                0f64
                            };

                            let end_time = 'rt: {
                                if let Ok(_end) = timeline_properties.EndTime() {
                                    let _duration: Duration = _end.into();
                                    break 'rt _duration.as_secs_f64();
                                }
                                0f64
                            };

                            end_time - start_time
                        },
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
                        // TODO: probably remove the unwrap hell from here
                        let stream = thumbnail.OpenReadAsync().unwrap().get().unwrap();

                        if stream.CanRead().unwrap() && stream.Size().unwrap() > 0 {
                            let result_buffer = {
                                let buffer = Streams::Buffer::Create(
                                    stream.Size().unwrap().try_into().unwrap(),
                                )
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

                            let data_reader = DataReader::FromBuffer(&result_buffer).unwrap();
                            let size = result_buffer.Length().unwrap();
                            let mut data: Vec<u8> = vec![0; size as usize];
                            data_reader.ReadBytes(data.as_mut()).unwrap();

                            stream.FlushAsync().unwrap().get().unwrap();
                            stream.Close().unwrap();

                            metadata.art_data = Some(ArtData {
                                data,
                                mimetype: stream.ContentType().unwrap().to_string(),
                            });
                        }
                    }

                    return Some(metadata);
            }
        }
    }
    None
}
