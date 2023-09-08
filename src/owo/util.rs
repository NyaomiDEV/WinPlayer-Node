use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};

use windows::{
    core::HSTRING,
    ApplicationModel,
    Media::{
        Control::{
            GlobalSystemMediaTransportControlsSession,
            GlobalSystemMediaTransportControlsSessionPlaybackInfo,
            GlobalSystemMediaTransportControlsSessionPlaybackStatus,
            GlobalSystemMediaTransportControlsSessionTimelineProperties,
        },
        MediaPlaybackAutoRepeatMode,
    },
    Storage::Streams::{self, DataReader, IRandomAccessStreamReference},
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

pub fn autorepeat_to_string(autorepeat: MediaPlaybackAutoRepeatMode) -> String {
    match autorepeat {
        MediaPlaybackAutoRepeatMode::List => String::from("List"),
        MediaPlaybackAutoRepeatMode::Track => String::from("Track"),
        _ => String::from("None"),
    }
}

pub fn playback_status_to_string(
    status: GlobalSystemMediaTransportControlsSessionPlaybackStatus,
) -> String {
    match status {
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => String::from("Playing"),
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => String::from("Paused"),
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => String::from("Stopped"),
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Changing => {
            String::from("Changing")
        }
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed => String::from("Closed"),
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Opened => String::from("Opened"),
        _ => String::from("Unknown"),
    }
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
                let title = info.Title().unwrap_or_default().to_string();

                let album = info.AlbumTitle().ok().map(|x| x.to_string());

                let album_artist = info.AlbumArtist().ok().map(|x| x.to_string());

                let album_artists = 'rt: {
                    if let Ok(artist) = info.AlbumArtist() {
                        break 'rt Some(vec![artist.to_string()]);
                    }
                    None
                };

                let artist = info.Artist().unwrap_or_default().to_string();

                let artists = vec![info.Artist().unwrap_or_default().to_string()];

                let art_data = 'rt: {
                    if let Ok(thumbnail) = info.Thumbnail() {
                        break 'rt get_cover_art_data(thumbnail);
                    }
                    None
                };

                let id = 'rt: {
                    let id = format!(
                        "{}{}{}{}",
                        album_artist.clone().unwrap_or(String::new()),
                        artist,
                        album.clone().unwrap_or(String::new()),
                        title
                    );
                    if !id.is_empty() {
                        let md5 = md5::compute(id);
                        break 'rt Some(format!("{:x}", md5).to_string());
                    }
                    None
                };

                let length = {
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
                };

                return Some(Metadata {
                    album,
                    album_artist,
                    album_artists,
                    artist,
                    artists,
                    art_data,
                    id,
                    length,
                    title,
                });
            }
        }
    }
    None
}

fn get_cover_art_data(thumbnail: IRandomAccessStreamReference) -> Option<ArtData> {
    if let Ok(_async) = thumbnail.OpenReadAsync() {
        if let Ok(stream) = _async.get() {
            let size = stream.Size().unwrap_or(0);
            let content_type = stream.ContentType().unwrap_or_default();

            if stream.CanRead().unwrap_or(false) && size > 0 {
                let result_buffer = 'rt: {
                    if let Ok(buffer) = Streams::Buffer::Create(size as u32) {
                        if let Ok(_async) = stream.ReadAsync(
                            &buffer,
                            size as u32,
                            Streams::InputStreamOptions::None,
                        ) {
                            if let Ok(result_buffer) = _async.get() {
                                break 'rt Some(result_buffer);
                            }
                        }
                    }
                    None
                };

                if let Some(result_buffer) = result_buffer {
                    let size = result_buffer.Length().unwrap_or(0);

                    if let Ok(data_reader) = DataReader::FromBuffer(&result_buffer) {
                        let mut data: Vec<u8> = vec![0; size as usize];
                        data_reader.ReadBytes(data.as_mut()).unwrap_or_default();

                        if let Ok(_async) = stream.FlushAsync() {
                            _async.get().unwrap_or_default();
                        }

                        stream.Close().unwrap_or_default();

                        return Some(ArtData {
                            data,
                            mimetype: content_type.to_string(),
                        });
                    }
                }
            }
        }
    }
    None
}
