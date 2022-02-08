#include "winPlayer.h"
#include "util.h"

#include <winrt/Windows.ApplicationModel.h>
#include <winrt/Windows.Graphics.Imaging.h>
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Foundation.Collections.h>
#include <winrt/Windows.Security.Cryptography.Core.h>
#include <winrt/Windows.Storage.Streams.h>
#include <winrt/Windows.System.h>

#include <chrono>
#include <functional>
#include <sstream>

#include <ppltasks.h>
#include <pplawait.h>

#include <sdkddkver.h>

// private
void Player::updatePlayers(){
	this->activePlayer.reset();
	this->players.clear();
	this->playbackInfoChangedHandlers.clear();
	this->mediaPropertiesChangedHandlers.clear();
	this->timelinePropertiesChangedHandlers.clear();
	if (this->callback.has_value()) (this->callback.value())();

	auto sessions = this->sessionManager->GetSessions();
	for(uint32_t i = 0; i < sessions.Size(); i++){
		const auto player = sessions.GetAt(i);
		auto AUMID = winrt::to_string(player.SourceAppUserModelId());
		this->addPlayer(AUMID, player);
	}
}

concurrency::task<std::string> Player::getPlayerName(GlobalSystemMediaTransportControlsSession player){
	auto playerName = winrt::to_string(player.SourceAppUserModelId());
#if WDK_NTDDI_VERSION >= NTDDI_WIN10_MN
	try {
		auto user = (co_await winrt::Windows::System::User::FindAllAsync()).GetAt(0);
		playerName = winrt::to_string(winrt::Windows::ApplicationModel::AppInfo::GetFromAppUserModelIdForUser(user, player.SourceAppUserModelId()).DisplayInfo().DisplayName());
	} catch (winrt::hresult_error e) {
		try {
			// possibly?
			playerName = winrt::to_string(winrt::Windows::ApplicationModel::AppInfo::GetFromAppUserModelId(player.SourceAppUserModelId()).DisplayInfo().DisplayName());
		}catch(winrt::hresult_error e) {
			// no dice :C
		}
	}
#endif
	if(winrt::to_string(player.SourceAppUserModelId()) == playerName && endsWith(playerName, ".exe")){
		playerName = playerName.substr(0, playerName.length() - 4);
	}
	co_return playerName;
}

void Player::addPlayer(std::string const AUMID, GlobalSystemMediaTransportControlsSession const& player){
	this->players.insert(std::pair(AUMID, player));
	this->registerPlayerEvents(AUMID, player);
	this->calculateActivePlayer(AUMID);
}

void Player::removePlayer(std::string const AUMID){
	this->players.erase(AUMID);
	this->calculateActivePlayer(std::nullopt);
}

void Player::registerPlayerEvents(std::string const AUMID, GlobalSystemMediaTransportControlsSession const& player){
	// Playing, Stopped, etc
	player.PlaybackInfoChanged(winrt::auto_revoke, [this](GlobalSystemMediaTransportControlsSession player, PlaybackInfoChangedEventArgs args){
		if(this->callback.has_value()) (this->callback.value())();
	}).swap(this->playbackInfoChangedHandlers[AUMID]);

	// Metadata
	player.MediaPropertiesChanged(winrt::auto_revoke, [this](GlobalSystemMediaTransportControlsSession player, MediaPropertiesChangedEventArgs args){
		if(this->callback.has_value()) (this->callback.value())();
	}).swap(this->mediaPropertiesChangedHandlers[AUMID]);

	// Seeked
	player.TimelinePropertiesChanged(winrt::auto_revoke, [this](GlobalSystemMediaTransportControlsSession player, TimelinePropertiesChangedEventArgs args){
		if(this->callback.has_value()) (this->callback.value())();
	}).swap(this->timelinePropertiesChangedHandlers[AUMID]);
}

void Player::calculateActivePlayer(std::optional<std::string> const preferred){
	std::optional<std::string> _activePlayer;
	std::map<std::string, std::optional<GlobalSystemMediaTransportControlsSession>>::iterator it = this->players.begin();

	while(it != this->players.end()){
		if(it->second->GetPlaybackInfo().PlaybackStatus() == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing){
			_activePlayer = it->first;
			break;
		}
		it++;
	}

	if(!_activePlayer.has_value() && preferred.has_value())
		_activePlayer = preferred.value();

	if(!_activePlayer.has_value() && this->players.size() > 0)
		_activePlayer = this->players.begin()->first;

	if(_activePlayer.has_value())
		this->activePlayer = _activePlayer.value();
	else
		this->activePlayer.reset();

	if(this->callback.has_value()) (this->callback.value())();
}

concurrency::task<std::optional<Metadata>> Player::getMetadata(GlobalSystemMediaTransportControlsSession player){
	auto timelineProperties = player.GetTimelineProperties();
	try	{
		auto info = co_await player.TryGetMediaPropertiesAsync();
		Metadata metadata;

		metadata.title = winrt::to_string(info.Title());
		metadata.album = winrt::to_string(info.AlbumTitle());
		metadata.artist = winrt::to_string(info.Artist());
		metadata.albumArtist = winrt::to_string(info.AlbumArtist());
		metadata.artists = {winrt::to_string(info.Artist())};
		metadata.albumArtists = {winrt::to_string(info.AlbumArtist())};
		metadata.length = std::chrono::duration_cast<std::chrono::milliseconds>(timelineProperties.EndTime() - timelineProperties.StartTime()).count() / 1000.0;

		auto id = winrt::to_hstring(metadata.albumArtist + metadata.artist + metadata.album + metadata.title);
		if(id.size() > 0){
			auto md5 = winrt::Windows::Security::Cryptography::Core::HashAlgorithmProvider::OpenAlgorithm(winrt::Windows::Security::Cryptography::Core::HashAlgorithmNames::Md5());
			winrt::Windows::Storage::Streams::IBuffer idBuf = winrt::Windows::Security::Cryptography::CryptographicBuffer::ConvertStringToBinary(
				id,
				winrt::Windows::Security::Cryptography::BinaryStringEncoding::Utf8
			);
			metadata.id = winrt::to_string(winrt::Windows::Security::Cryptography::CryptographicBuffer::EncodeToHexString(md5.HashData(idBuf)));
		}

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
		}
		co_return metadata;
	} catch (winrt::hresult_error e) {
		// oof
		co_return {};
	}
}

Capabilities Player::getCapabilities(GlobalSystemMediaTransportControlsSession const& player){
	auto controls = player.GetPlaybackInfo().Controls();

	Capabilities caps;

	caps.canPlayPause = controls.IsPlayEnabled() || controls.IsPauseEnabled();
	caps.canGoNext = controls.IsNextEnabled();
	caps.canGoPrevious = controls.IsPreviousEnabled();
	caps.canSeek = controls.IsPlaybackPositionEnabled() && std::chrono::duration_cast<std::chrono::milliseconds>(player.GetTimelineProperties().EndTime()).count() != 0;

	caps.canControl = caps.canPlayPause || caps.canGoNext || caps.canGoPrevious || caps.canSeek;

	return caps;
}

// public
Player::Player(){
	this->sessionManager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().get();
	this->sessionManager->SessionsChanged([this](GlobalSystemMediaTransportControlsSessionManager, SessionsChangedEventArgs){
		this->updatePlayers();
	});
	this->updatePlayers();
}

void Player::setCallback(CallbackFn const callback){
	this->callback = callback;
	(this->callback.value())();
}

concurrency::task<std::optional<Update>> Player::getUpdate(){
	if(!this->activePlayer.has_value())
		return {};

	auto player = this->players[this->activePlayer.value()];
	auto playbackInfo = player->GetPlaybackInfo();
	auto timelineProperties = player->GetTimelineProperties();

	Update update;
	// Fire the async metadata retrieval
	auto metadata = this->getMetadata(*player);
	update.capabilities = this->getCapabilities(*player);

	update.app = winrt::to_string(player->SourceAppUserModelId());
	update.appName = co_await this->getPlayerName(*player);

	if(playbackInfo.IsShuffleActive()){
		update.shuffle = playbackInfo.IsShuffleActive().Value();
	}else{
		update.shuffle = false;
	}

	update.volume = -1; // hardcode it for now

	update.elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(timelineProperties.Position() - timelineProperties.StartTime()).count() / 1000.0;


	switch(playbackInfo.PlaybackStatus()) {
		case GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing:
			update.status = "Playing";
			break;
		case GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused:
			update.status = "Paused";
			break;
		case GlobalSystemMediaTransportControlsSessionPlaybackStatus::Changing:
		case GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed:
		case GlobalSystemMediaTransportControlsSessionPlaybackStatus::Opened:
		case GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped:
		default:
			update.status = "Stopped";
			break;
	}

	if(playbackInfo.AutoRepeatMode()){
		switch(playbackInfo.AutoRepeatMode().Value()){
			case winrt::Windows::Media::MediaPlaybackAutoRepeatMode::List:
				update.loop = "Playlist";
				break;
			case winrt::Windows::Media::MediaPlaybackAutoRepeatMode::Track:
				update.loop = "Track";
				break;
			case winrt::Windows::Media::MediaPlaybackAutoRepeatMode::None:
			default:
				update.loop = "None";
				break;
		}
	}else{
		update.loop = "None";
	}

	// get metadata now
	update.metadata = co_await metadata;

	co_return update;
}

void Player::Play(){
	if(this->activePlayer.has_value()) FireAndForget(this->players[this->activePlayer.value()]->TryPlayAsync());
};

void Player::Pause(){
	if(this->activePlayer.has_value()) FireAndForget(this->players[this->activePlayer.value()]->TryPauseAsync());
}

void Player::PlayPause(){
	if(this->activePlayer.has_value()) FireAndForget(this->players[this->activePlayer.value()]->TryTogglePlayPauseAsync());
}

void Player::Stop(){
	if(this->activePlayer.has_value()) FireAndForget(this->players[this->activePlayer.value()]->TryStopAsync());
}

void Player::Next(){
	if(this->activePlayer.has_value()) FireAndForget(this->players[this->activePlayer.value()]->TrySkipNextAsync());
}

void Player::Previous(){
	if(this->activePlayer.has_value()) FireAndForget(this->players[this->activePlayer.value()]->TrySkipPreviousAsync());
}

void Player::Shuffle(){
	if(!this->activePlayer.has_value())
		return;
	
	auto player = this->players[this->activePlayer.value()];
	auto playbackInfo = player->GetPlaybackInfo();
	bool isShuffle = false;
	if(playbackInfo.IsShuffleActive())
		isShuffle = playbackInfo.IsShuffleActive().Value();
	FireAndForget(player->TryChangeShuffleActiveAsync(!isShuffle));
}

void Player::Repeat(){
	if(!this->activePlayer.has_value())
		return;

	auto player = this->players[this->activePlayer.value()];
	auto playbackInfo = player->GetPlaybackInfo();
	auto repeat = winrt::Windows::Media::MediaPlaybackAutoRepeatMode::None;
	if(playbackInfo.AutoRepeatMode())
		repeat = playbackInfo.AutoRepeatMode().Value();

	switch (repeat){
		case winrt::Windows::Media::MediaPlaybackAutoRepeatMode::List:
			repeat = winrt::Windows::Media::MediaPlaybackAutoRepeatMode::Track;
			break;
		case winrt::Windows::Media::MediaPlaybackAutoRepeatMode::Track:
			repeat = winrt::Windows::Media::MediaPlaybackAutoRepeatMode::None;
			break;
		case winrt::Windows::Media::MediaPlaybackAutoRepeatMode::None:
			repeat = winrt::Windows::Media::MediaPlaybackAutoRepeatMode::List;
			break;
	}

	FireAndForget(player->TryChangeAutoRepeatModeAsync(repeat));
}

void Player::Seek(int const offsetUs){
	if(!this->activePlayer.has_value())
		return;
	
	auto player = this->players[this->activePlayer.value()];
    winrt::Windows::Foundation::TimeSpan offset = std::chrono::microseconds(offsetUs);
    FireAndForget(player->TryChangePlaybackPositionAsync((player->GetTimelineProperties().Position() + offset).count()));
}

void Player::SeekPercentage(float const percentage){
	if(!this->activePlayer.has_value())
		return;

	auto player = this->players[this->activePlayer.value()];
	float length = std::chrono::duration_cast<std::chrono::milliseconds>(player->GetTimelineProperties().EndTime() - player->GetTimelineProperties().StartTime()).count() / 1000.0;
	this->SetPosition(length * percentage);
}

void Player::SetPosition(float const positionS){
	if(!this->activePlayer.has_value())
		return;

	auto player = this->players[this->activePlayer.value()];
	winrt::Windows::Foundation::TimeSpan position = std::chrono::milliseconds(static_cast<int>(positionS * 1000));
    FireAndForget(player->TryChangePlaybackPositionAsync((player->GetTimelineProperties().StartTime() + position).count()));
}

float Player::GetPosition(){
	if(!this->activePlayer.has_value())
		return 0.0;

	auto player = this->players[this->activePlayer.value()];
	auto timelineProperties = player->GetTimelineProperties();

	if (std::chrono::duration_cast<std::chrono::milliseconds>(timelineProperties.EndTime()).count() == 0)
		return 0.0;

	auto position = timelineProperties.Position();

	if (player->GetPlaybackInfo().PlaybackStatus() == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing){
			auto timeFromLastUpdate = winrt::clock::now().time_since_epoch() - timelineProperties.LastUpdatedTime().time_since_epoch();
			position += timeFromLastUpdate;
		}

/* 	std::cout
		<< std::chrono::duration_cast<std::chrono::milliseconds>(timelineProperties.StartTime()).count() << std::endl
		<< std::chrono::duration_cast<std::chrono::milliseconds>(timelineProperties.EndTime()).count() << std::endl
		<< std::chrono::duration_cast<std::chrono::milliseconds>(timelineProperties.Position()).count() << std::endl
		<< std::chrono::duration_cast<std::chrono::milliseconds>(timelineProperties.LastUpdatedTime().time_since_epoch()).count() << std::endl; */

	return std::chrono::duration_cast<std::chrono::milliseconds>(position - timelineProperties.StartTime()).count() / 1000.0;
}

float Player::GetVolume(){
	// not supported :C
	return -1;
}

void Player::SetVolume(float const volume){
	// not supported :C
}