#ifndef WINPLAYER_UTIL_H
#define WINPLAYER_UTIL_H

#include <string_view>
#include <winrt/base.h>
#include <winrt/Windows.Foundation.h>

winrt::fire_and_forget FireAndForget(winrt::Windows::Foundation::IAsyncOperation<bool> operation){
	co_await operation;
}

static bool endsWith(std::string_view str, std::string_view suffix){
    return str.size() >= suffix.size() && 0 == str.compare(str.size()-suffix.size(), suffix.size(), suffix);
}

#endif // WINPLAYER_UTIL_H