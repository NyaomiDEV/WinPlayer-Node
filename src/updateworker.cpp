#include "updateworker.h"

#include <chrono>

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
	this->currentUpdate = this->player->getUpdate().get();
}

void UpdateWorker::OnOK() {
	auto env = promise.Env();
	if (currentUpdate.has_value()){
		Napi::Object jsUpdate = Napi::Object::New(env);

		jsUpdate.Set("provider", Napi::String::New(env, "WinPlayer"));
		jsUpdate.Set("status", Napi::String::New(env, currentUpdate->status));
		jsUpdate.Set("loop", Napi::String::New(env, currentUpdate->loop));
		jsUpdate.Set("app", stringOrUndefined(env, currentUpdate->app));
		jsUpdate.Set("appName", stringOrUndefined(env, currentUpdate->appName));
		jsUpdate.Set("shuffle", Napi::Boolean::New(env, currentUpdate->shuffle));
		jsUpdate.Set("volume", Napi::Number::New(env, currentUpdate->volume));

		Napi::Object jsPosition = Napi::Object::New(env);
		jsPosition.Set("howMuch", Napi::Number::New(env, currentUpdate->elapsed));
		jsPosition.Set("when",
			Napi::Date::New(
				env,
				std::chrono::duration_cast<std::chrono::milliseconds>(
					std::chrono::system_clock::now().time_since_epoch()
				).count()
			)
		);
		jsUpdate.Set("elapsed", jsPosition);

		// Capabilities
		Napi::Object jsCaps = Napi::Object::New(env);
		jsCaps.Set("canControl", Napi::Boolean::New(env, currentUpdate->capabilities.canControl));
		jsCaps.Set("canPlayPause", Napi::Boolean::New(env, currentUpdate->capabilities.canPlayPause));
		jsCaps.Set("canGoNext", Napi::Boolean::New(env, currentUpdate->capabilities.canGoNext));
		jsCaps.Set("canGoPrevious", Napi::Boolean::New(env, currentUpdate->capabilities.canGoPrevious));
		jsCaps.Set("canSeek", Napi::Boolean::New(env, currentUpdate->capabilities.canSeek));
		jsUpdate.Set("capabilities", jsCaps);

		// Metadata
		if (currentUpdate->metadata.has_value()){
			Napi::Object jsMetadata = Napi::Object::New(env);

			jsMetadata.Set("id", stringOrUndefined(env, currentUpdate->metadata->id));
			jsMetadata.Set("title", Napi::String::New(env, currentUpdate->metadata->title));
			jsMetadata.Set("artist", Napi::String::New(env, currentUpdate->metadata->artist));
			jsMetadata.Set("album", stringOrUndefined(env, currentUpdate->metadata->album));
			jsMetadata.Set("albumArtist", stringOrUndefined(env, currentUpdate->metadata->albumArtist));
			jsMetadata.Set("length", Napi::Number::New(env, currentUpdate->metadata->length));

 			Napi::Array jsArtists = Napi::Array::New(env),
						jsAlbumArtists = Napi::Array::New(env);
			for (int i = 0; i < currentUpdate->metadata->artists.size(); i++){
				if (currentUpdate->metadata->artists[i].size() > 0)
					jsArtists.Set(i, currentUpdate->metadata->artists[i]);
			}
			for (int i = 0; i < currentUpdate->metadata->albumArtists.size(); i++){
				if (currentUpdate->metadata->albumArtists[i].size() > 0)
					jsAlbumArtists.Set(i, currentUpdate->metadata->albumArtists[i]);
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

				jsUpdate.Set("metadata", jsMetadata);
			}
		}

		promise.Resolve(jsUpdate);
		return;
	}

	promise.Resolve(env.Null());
}