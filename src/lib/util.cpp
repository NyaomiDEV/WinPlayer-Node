#include <winrt/base.h>
#include <winrt/Windows.Foundation.h>

winrt::fire_and_forget FireAndForget(winrt::Windows::Foundation::IAsyncOperation<bool> operation){
	co_await operation;
}