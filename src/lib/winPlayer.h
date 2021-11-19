#ifndef WINPLAYER_H
#define WINPLAYER_H

#include <winrt/Windows.ApplicationModel.h>
#include <winrt/Windows.Media.Control.h>
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Foundation.Collections.h>
#include <winrt/Windows.Storage.Streams.h>
#include <winrt/Windows.System.h>

#include <chrono>
#include <functional>

#include <sdkddkver.h>

#include "types.h"

using namespace winrt::Windows::Media::Control;

using CallbackFn = std::function<void(void)>;

class Player {
	private:
		std::optional<GlobalSystemMediaTransportControlsSessionManager> sessionManager;

		std::map<std::wstring, std::optional<GlobalSystemMediaTransportControlsSession>> players;
		std::optional<std::wstring> activePlayer;
		std::optional<CallbackFn> callback;

		std::map<std::wstring, GlobalSystemMediaTransportControlsSession::PlaybackInfoChanged_revoker> playbackInfoChangedHandlers;
		std::map<std::wstring, GlobalSystemMediaTransportControlsSession::MediaPropertiesChanged_revoker> mediaPropertiesChangedHandlers;
		std::map<std::wstring, GlobalSystemMediaTransportControlsSession::TimelinePropertiesChanged_revoker> timelinePropertiesChangedHandlers;

		void updatePlayers();
		std::wstring getPlayerName(GlobalSystemMediaTransportControlsSession const& player);
		void addPlayer(std::wstring const AUMID, GlobalSystemMediaTransportControlsSession const& player);
		void removePlayer(std::wstring const AUMID);
		void registerPlayerEvents(std::wstring const AUMID, GlobalSystemMediaTransportControlsSession const& player);
		void calculateActivePlayer(std::optional<std::wstring> const preferred);
		std::optional<Metadata> getMetadata(GlobalSystemMediaTransportControlsSession const& player);
		Capabilities getCapabilities(GlobalSystemMediaTransportControlsSession const& player);
	public:
		Player();
		void setCallback(CallbackFn const callback);
		std::optional<Update> getUpdate();
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
