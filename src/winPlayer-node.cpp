#include <napi.h>
#include <rpc.h>
#include <chrono>
#include <iostream>

using namespace std::chrono_literals;

#include "lib/winPlayer.h"

std::optional<Napi::Function> jsCallback;
std::optional<Player> winPlayer;
std::optional<winrt::Windows::System::DispatcherQueueController> controller;
std::optional<CallbackFn> cb;

Napi::Value setCallback(const Napi::CallbackInfo& info){
	jsCallback = info[0].As<Napi::Function>();
	return Napi::Boolean::New(info.Env(), true);
}

winrt::fire_and_forget RunAsync(winrt::Windows::System::DispatcherQueue queue){
    co_await winrt::resume_foreground(queue, winrt::Windows::System::DispatcherQueuePriority::High);
}

Napi::Object Init(Napi::Env env, Napi::Object exports){
    try{
        winrt::init_apartment();
    }catch (winrt::hresult_error hresult){
        // electron already initialized the COM library
        if (hresult.code() != RPC_E_CHANGED_MODE){
            wprintf(L"Failed initializing apartment: %d %s", hresult.code().value,
                    hresult.message().c_str());
            Napi::TypeError::New(env, "Failed initializing apartment").ThrowAsJavaScriptException();
            return exports;
        }
    }
	controller = winrt::Windows::System::DispatcherQueueController::CreateOnDedicatedThread();
	RunAsync(controller->DispatcherQueue());
	
	winPlayer = Player();

	cb = []() -> void {

		auto update = winPlayer->getUpdate();
		if(update.has_value()){
			std::wcout << L"------" << std::endl;
			std::wcout << L"STATUS: " << update->status << std::endl;
			std::wcout << L"LOOP: " << update->loop << std::endl;
			std::wcout << L"SHUFFLE: " << update->shuffle << std::endl;
			std::wcout << L"VOLUME (-1): " << update->volume << std::endl;
			std::wcout << L"ELAPSED: " << update->elapsed << std::endl;
			std::wcout << L"APP: " << update->app << std::endl;
			std::wcout << L"APP NAME: " << update->appName << std::endl;
			std::wcout << L"------" << std::endl;
			std::wcout << L"CAN CONTROL: " << update->capabilities.canControl << std::endl;
			std::wcout << L"CAN PLAY OR PAUSE: " << update->capabilities.canPlayPause << std::endl;
			std::wcout << L"CAN GO PREVIOUS: " << update->capabilities.canGoPrevious << std::endl;
			std::wcout << L"CAN GO NEXT: " << update->capabilities.canGoNext << std::endl;
			std::wcout << L"CAN SEEK: " << update->capabilities.canSeek << std::endl;
			std::wcout << L"------" << std::endl;
			std::wcout << L"ID: " << update->metadata.id << std::endl;
			std::wcout << L"TITLE: " << update->metadata.title << std::endl;
			std::wcout << L"ARTIST: " << update->metadata.artist << std::endl;
			std::wcout << L"ALBUM: " << update->metadata.album << std::endl;
			std::wcout << L"ALBUM ARTIST: " << update->metadata.albumArtist << std::endl;
			std::wcout << L"LENGTH: " << update->metadata.length << std::endl;
			std::wcout << L"ART TYPE: " << (update->metadata.artData.has_value() ? update->metadata.artData->type : L"NULL") << std::endl;
			std::wcout << L"ART SIZE: " << (update->metadata.artData.has_value() ? update->metadata.artData->size : 0) << std::endl;
		}
		if(jsCallback.has_value()) jsCallback->Call({});
	};

	winPlayer->setCallback(*cb);

	exports.Set(Napi::String::New(env, "setCallback"), Napi::Function::New(env, setCallback));
	return exports;
}

NODE_API_MODULE(NODE_GYP_MODULE_NAME, Init);