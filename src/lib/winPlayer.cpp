#include "winPlayer.h"

// private
void Player::updatePlayers(){
	this->activePlayer.reset();
	this->players.clear();
	this->playbackInfoChangedHandlers.clear();
	this->mediaPropertiesChangedHandlers.clear();
	this->timelinePropertiesChangedHandlers.clear();
	auto sessions = this->sessionManager->GetSessions();
	for(uint32_t i = 0; i < sessions.Size(); i++){
		const auto player = sessions.GetAt(i);
		auto AUMID = winrt::to_string(player.SourceAppUserModelId());
		this->addPlayer(AUMID, player);
	}
}

std::string Player::getPlayerName(GlobalSystemMediaTransportControlsSession const& player){
	auto playerName = player.SourceAppUserModelId();
#if WDK_NTDDI_VERSION >= NTDDI_WIN10_MN
	try {
		auto user = winrt::Windows::System::User::FindAllAsync().get().GetAt(0);
		playerName = winrt::Windows::ApplicationModel::AppInfo::GetFromAppUserModelIdForUser(user, playerName).DisplayInfo().DisplayName();
	} catch (winrt::hresult_error e) {
		try {
			// possibly?
			playerName = winrt::Windows::ApplicationModel::AppInfo::GetFromAppUserModelId(playerName).DisplayInfo().DisplayName();
		}catch(winrt::hresult_error e) {
			// no dice :C
		}
	}
#endif
	return winrt::to_string(playerName);
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

std::optional<Metadata> Player::getMetadata(GlobalSystemMediaTransportControlsSession const& player){
	auto timelineProperties = player.GetTimelineProperties();
	try{
		auto info = player.TryGetMediaPropertiesAsync().get();
		Metadata metadata;

		metadata.title = winrt::to_string(info.Title());
		metadata.album = winrt::to_string(info.AlbumTitle());
		metadata.artist = winrt::to_string(info.Artist());
		metadata.albumArtist = winrt::to_string(info.AlbumArtist());
		metadata.artists = {winrt::to_string(info.Artist())};
		metadata.albumArtists = {winrt::to_string(info.AlbumArtist())};
		metadata.length = std::chrono::duration_cast<std::chrono::milliseconds>(timelineProperties.EndTime() - timelineProperties.StartTime()).count() / 1000.0;
		metadata.id = metadata.albumArtist + ":" + metadata.artist + ":" + metadata.album + ":" + metadata.title + ":" + std::to_string(metadata.length);

		auto thumbnail = info.Thumbnail();
		if(thumbnail) {
			auto stream = thumbnail.OpenReadAsync().get();
			if(stream && stream.CanRead()) {
				winrt::Windows::Storage::Streams::IBuffer data = winrt::Windows::Storage::Streams::Buffer(stream.Size());
				data = stream.ReadAsync(data, stream.Size(), winrt::Windows::Storage::Streams::InputStreamOptions::None).get();
				metadata.artData.data = data.data();
				metadata.artData.size = data.Capacity();
				metadata.artData.type = winrt::to_string(stream.ContentType());
			}
		}else{
			metadata.artData.data = nullptr;
			metadata.artData.size = 0;
			metadata.artData.type = "NULL";
		}

		return metadata;
	}catch(winrt::hresult_error e){
		// oof
		return {};
	}
}

Capabilities Player::getCapabilities(GlobalSystemMediaTransportControlsSession const& player){
	auto controls = player.GetPlaybackInfo().Controls();

	Capabilities caps;

	caps.canPlayPause = controls.IsPlayEnabled() || controls.IsPauseEnabled();
	caps.canGoNext = controls.IsNextEnabled();
	caps.canGoPrevious = controls.IsPreviousEnabled();
	caps.canSeek = controls.IsPlaybackPositionEnabled();

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

std::optional<Update> Player::getUpdate(){
	if(!this->activePlayer.has_value())
		return {};

	auto player = this->players[this->activePlayer.value()];
	auto playbackInfo = player->GetPlaybackInfo();
	auto timelineProperties = player->GetTimelineProperties();

	Update update;

	update.app = winrt::to_string(player->SourceAppUserModelId());
	update.appName = this->getPlayerName(*player);

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

	update.metadata = this->getMetadata(*player);
	update.capabilities = this->getCapabilities(*player);

	return update;
}

void Player::Play(){
	if(this->activePlayer.has_value()) this->players[this->activePlayer.value()]->TryPlayAsync().get();
};

void Player::Pause(){
	if(this->activePlayer.has_value()) this->players[this->activePlayer.value()]->TryPauseAsync().get();
}

void Player::PlayPause(){
	if(this->activePlayer.has_value()) this->players[this->activePlayer.value()]->TryTogglePlayPauseAsync().get();
}

void Player::Stop(){
	if(this->activePlayer.has_value()) this->players[this->activePlayer.value()]->TryStopAsync().get();
}

void Player::Next(){
	if(this->activePlayer.has_value()) this->players[this->activePlayer.value()]->TrySkipNextAsync().get();
}

void Player::Previous(){
	if(this->activePlayer.has_value()) this->players[this->activePlayer.value()]->TrySkipPreviousAsync().get();
}

void Player::Shuffle(){
	if(this->activePlayer.has_value()){
		auto player = this->players[this->activePlayer.value()];
		auto playbackInfo = player->GetPlaybackInfo();
		bool isShuffle = false;
		if(playbackInfo.IsShuffleActive())
			isShuffle = playbackInfo.IsShuffleActive().Value();
		player->TryChangeShuffleActiveAsync(!isShuffle).get();
	}
}

void Player::Repeat(){
	if(this->activePlayer.has_value()){
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

		player->TryChangeAutoRepeatModeAsync(repeat).get();
	}
}

void Player::Seek(int const offsetUs){
	if(!this->activePlayer.has_value())
		return;
	
	auto player = this->players[this->activePlayer.value()];
    winrt::Windows::Foundation::TimeSpan offset = std::chrono::microseconds(offsetUs);
    player->TryChangePlaybackPositionAsync((player->GetTimelineProperties().Position() + offset).count()).get();
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
    player->TryChangePlaybackPositionAsync((player->GetTimelineProperties().StartTime() + position).count()).get();
}

float Player::GetPosition(){
	if(!this->activePlayer.has_value())
		return 0.0;

	auto player = this->players[this->activePlayer.value()];
	auto timelineProperties = player->GetTimelineProperties();
	return std::chrono::duration_cast<std::chrono::milliseconds>(timelineProperties.Position() - timelineProperties.StartTime()).count() / 1000.0;
}

float Player::GetVolume(){
	// not supported :C
	return -1;
}

void Player::SetVolume(float const volume){
	// not supported :C
}