#ifndef WINPLAYER_H
#define WINPLAYER_H

#include <winrt/base.h>
#include <winrt/Windows.Media.Control.h>

#include <chrono>
#include <functional>
#include <sstream>

#include <ppltasks.h>
#include <pplawait.h>

#include "types.h"

using namespace winrt::Windows::Media::Control;

using CallbackFn = std::function<void(void)>;

class Player {
	private:
		std::optional<GlobalSystemMediaTransportControlsSessionManager> sessionManager;

		std::map<std::string, std::optional<GlobalSystemMediaTransportControlsSession>> players;
		std::optional<std::string> activePlayer;
		std::optional<CallbackFn> callback;

		std::map<std::string, GlobalSystemMediaTransportControlsSession::PlaybackInfoChanged_revoker> playbackInfoChangedHandlers;
		std::map<std::string, GlobalSystemMediaTransportControlsSession::MediaPropertiesChanged_revoker> mediaPropertiesChangedHandlers;
		std::map<std::string, GlobalSystemMediaTransportControlsSession::TimelinePropertiesChanged_revoker> timelinePropertiesChangedHandlers;

		void updatePlayers();
		concurrency::task<std::string> getPlayerName(GlobalSystemMediaTransportControlsSession player);
		void addPlayer(std::string const AUMID, GlobalSystemMediaTransportControlsSession const& player);
		void removePlayer(std::string const AUMID);
		void registerPlayerEvents(std::string const AUMID, GlobalSystemMediaTransportControlsSession const& player);
		void calculateActivePlayer(std::optional<std::string> const preferred);
		concurrency::task<std::optional<Metadata>> getMetadata(GlobalSystemMediaTransportControlsSession player);
		Capabilities getCapabilities(GlobalSystemMediaTransportControlsSession const& player);
	public:
		Player();
		void setCallback(CallbackFn const callback);
		concurrency::task<std::optional<Update>> getUpdate();
		void Play();
		void Pause();
		void PlayPause();
		void Stop();
		void Next();
		void Previous();
		void Shuffle();
		void Repeat();
		void Seek(int const offsetUs);
		void SeekPercentage(float const percentage);
		float GetPosition();
		void SetPosition(float const positionS);
		float GetVolume();
		void SetVolume(float const volume);
};

#endif // WINPLAYER_H
