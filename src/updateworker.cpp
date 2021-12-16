#include "updateworker.h"

#include <iostream>

Napi::Value stringOrUndefined(const Napi::Env &env, std::string value){
	if (value.size() > 0)
		return Napi::String::New(env, value);
	return env.Undefined();
}

UpdateWorker::~UpdateWorker() {}

UpdateWorker::UpdateWorker(Player &player, Napi::Promise::Deferred const& promise)
: Napi::AsyncWorker(getFakeCallback(promise.Env()).Value()), promise(promise) {
	this->player = &player;
}

void UpdateWorker::Execute() {
	this->currentUpdate = *this->player->getUpdate().get();
}

void UpdateWorker::OnOK() {
	auto env = promise.Env();
	if (currentUpdate.has_value()){
		Napi::Object jsUpdate = Napi::Object::New(env);

		jsUpdate.Set("provider", Napi::String::New(env, "WinPlayer"));
		jsUpdate.Set("status", stringOrUndefined(env, currentUpdate->status));
		jsUpdate.Set("loop", stringOrUndefined(env, currentUpdate->loop));
		jsUpdate.Set("app", stringOrUndefined(env, currentUpdate->app));
		jsUpdate.Set("appName", stringOrUndefined(env, currentUpdate->appName));
		jsUpdate.Set("shuffle", Napi::Boolean::New(env, currentUpdate->shuffle));
		jsUpdate.Set("volume", Napi::Number::New(env, currentUpdate->volume));
		jsUpdate.Set("elapsed", Napi::Number::New(env, currentUpdate->elapsed));

		// Capabilities
		Napi::Object jsCaps = Napi::Object::New(env);
		jsCaps.Set("canControl", Napi::Boolean::New(env, currentUpdate->capabilities.canControl));
		jsCaps.Set("canPlayPause", Napi::Boolean::New(env, currentUpdate->capabilities.canPlayPause));
		jsCaps.Set("canGoNext", Napi::Boolean::New(env, currentUpdate->capabilities.canGoNext));
		jsCaps.Set("canGoPrevious", Napi::Boolean::New(env, currentUpdate->capabilities.canGoPrevious));
		jsCaps.Set("canSeek", Napi::Boolean::New(env, currentUpdate->capabilities.canSeek));
		jsUpdate.Set("capabilities", jsCaps);

		// Metadata
		Napi::Object jsMetadata = Napi::Object::New(env);
		if (currentUpdate->metadata.has_value()){
			jsMetadata.Set("id", stringOrUndefined(env, currentUpdate->metadata->id));
			jsMetadata.Set("title", stringOrUndefined(env, currentUpdate->metadata->title));
			jsMetadata.Set("artist", stringOrUndefined(env, currentUpdate->metadata->artist));
			jsMetadata.Set("album", stringOrUndefined(env, currentUpdate->metadata->album));
			jsMetadata.Set("albumArtist", stringOrUndefined(env, currentUpdate->metadata->albumArtist));
			jsMetadata.Set("length", Napi::Number::New(env, currentUpdate->metadata->length));

 			Napi::Array jsArtists = Napi::Array::New(env),
						jsAlbumArtists = Napi::Array::New(env);
			for (int i = 0; i < currentUpdate->metadata->artists.size(); i++){
				auto value = stringOrUndefined(env, currentUpdate->metadata->artists[i]);
				if (value != env.Undefined())
					jsArtists.Set(i, value);
			}
			for (int i = 0; i < currentUpdate->metadata->albumArtists.size(); i++){
				auto value = stringOrUndefined(env, currentUpdate->metadata->albumArtists[i]);
				if (value != env.Undefined())
					jsAlbumArtists.Set(i, value);
			}
			jsMetadata.Set("artists", jsArtists);
			jsMetadata.Set("albumArtists", jsAlbumArtists);

 			if (currentUpdate->metadata->artData.data.size() > 0){
				Napi::Object jsArtData = Napi::Object::New(env);

				Napi::Array jsArtType = Napi::Array::New(env, currentUpdate->metadata->artData.type.size());
				for (int i = 0; i < currentUpdate->metadata->artData.type.size(); i++)
					jsArtType.Set(i, Napi::String::New(env, currentUpdate->metadata->artData.type[i]));
				jsArtData.Set("type", jsArtType);

				Napi::Buffer buf = Napi::Buffer<uint8_t>::Copy(env, currentUpdate->metadata->artData.data.data(), currentUpdate->metadata->artData.data.size());
				jsArtData.Set("data", buf);

				jsMetadata.Set("artData", jsArtData);
			}
		}else{
			jsMetadata.Set("id", env.Undefined());
			jsMetadata.Set("title", env.Undefined());
			jsMetadata.Set("artist", env.Undefined());
			jsMetadata.Set("album", env.Undefined());
			jsMetadata.Set("albumArtist", env.Undefined());
			jsMetadata.Set("length", env.Undefined());
			jsMetadata.Set("artists", env.Undefined());
			jsMetadata.Set("albumArtists", env.Undefined());
		}

		jsUpdate.Set("metadata", jsMetadata);
		promise.Resolve(jsUpdate);
		return;
	}

	promise.Resolve(env.Null());
}