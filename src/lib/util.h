#ifndef WINPLAYER_UTIL_H
#define WINPLAYER_UTIL_H

#include <winrt/base.h>
#include <winrt/Windows.Foundation.h>

winrt::fire_and_forget FireAndForget(winrt::Windows::Foundation::IAsyncOperation<bool> operation){
	co_await operation;
}

#endif // WINPLAYER_UTIL_H