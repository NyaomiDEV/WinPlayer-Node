#ifndef WINPLAYER_TYPES_H
#define WINPLAYER_TYPES_H

#include <map>
#include <string>
#include <vector>

struct ArtData {
	uint8_t* data;
	int size;
	std::wstring type;
};

struct Metadata {
	std::wstring id;
	std::wstring title;
	std::wstring artist;
	std::vector<std::wstring> artists;
	std::wstring album;
	std::wstring albumArtist;
	std::vector<std::wstring> albumArtists;
	ArtData artData;
	float length;
};

struct Capabilities {
	bool canControl;
	bool canPlayPause;
	bool canGoNext;
	bool canGoPrevious;
	bool canSeek;
};

struct Update {
	std::optional<Metadata> metadata;
	Capabilities capabilities;
	std::wstring status;
	std::wstring loop;
	bool shuffle;
	float volume;
	float elapsed;
	std::wstring app;
	std::wstring appName;
};

#endif // WINPLAYER_TYPES_H
