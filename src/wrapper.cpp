#include "wrapper.h"

Napi::Object WrappedPlayer::Init(Napi::Env env, Napi::Object exports){
	Napi::Function func = DefineClass(env, "Player", {
		InstanceMethod("getUpdate", &WrappedPlayer::getUpdate),
		InstanceMethod("Play", &WrappedPlayer::Play),
		InstanceMethod("Pause", &WrappedPlayer::Pause),
		InstanceMethod("Stop", &WrappedPlayer::Stop),
		InstanceMethod("Next", &WrappedPlayer::Next),
		InstanceMethod("Previous", &WrappedPlayer::Previous),
		InstanceMethod("Shuffle", &WrappedPlayer::Shuffle),
		InstanceMethod("Repeat", &WrappedPlayer::Repeat),
		InstanceMethod("Seek", &WrappedPlayer::Seek),
		InstanceMethod("SeekPercentage", &WrappedPlayer::SeekPercentage),
		InstanceMethod("GetPosition", &WrappedPlayer::GetPosition),
		InstanceMethod("SetPosition", &WrappedPlayer::SetPosition),
		InstanceMethod("SetVolume", &WrappedPlayer::SetVolume),
	});

	Napi::FunctionReference* constructor = new Napi::FunctionReference();
	*constructor = Napi::Persistent(func);
	env.SetInstanceData(constructor);

	exports.Set("Player", func);
	return exports;
}

WrappedPlayer::WrappedPlayer(const Napi::CallbackInfo &info) : Napi::ObjectWrap<WrappedPlayer>(info) {
	// THIS WAS HELL
	// ALL MY TEARS ARE HERE
	// THREE DAYS AND IT WAS THIS SIMPLE
	// PLEASE KILL MYSELF
	auto tsfn = Napi::ThreadSafeFunction::New(
		info.Env(),
		info[0].As<Napi::Function>(),
		"Callback",
		0,
		1
	);
	this->_player.setCallback([tsfn](){
		auto callback = []( Napi::Env env, Napi::Function jsCallback) { jsCallback.Call({}); };
		tsfn.BlockingCall(callback);
	});
}

Napi::Value WrappedPlayer::getUpdate(const Napi::CallbackInfo &info){
	auto env = info.Env();
	auto update = this->_player.getUpdate();
	if(update.has_value()){
		Napi::Object jsUpdate = Napi::Object::New(env);

		jsUpdate.Set("provider", Napi::String::New(env, "WinPlayer"));
		jsUpdate.Set("status", Napi::String::New(env, std::u16string(update->status.begin(), update->status.end())));
		jsUpdate.Set("loop", Napi::String::New(env, std::u16string(update->loop.begin(), update->loop.end())));
		jsUpdate.Set("app", Napi::String::New(env, std::u16string(update->app.begin(), update->app.end())));
		jsUpdate.Set("appName", Napi::String::New(env, std::u16string(update->appName.begin(), update->appName.end())));
		jsUpdate.Set("shuffle", Napi::Boolean::New(env, update->shuffle));
		jsUpdate.Set("volume", Napi::Number::New(env, update->volume));
		jsUpdate.Set("elapsed", Napi::Number::New(env, update->elapsed));

		// Capabilities
		Napi::Object jsCaps = Napi::Object::New(env);
		jsCaps.Set("canControl", Napi::Boolean::New(env, update->capabilities.canControl));
		jsCaps.Set("canPlayPause", Napi::Boolean::New(env, update->capabilities.canPlayPause));
		jsCaps.Set("canGoNext", Napi::Boolean::New(env, update->capabilities.canGoNext));
		jsCaps.Set("canGoPrevious", Napi::Boolean::New(env, update->capabilities.canGoPrevious));
		jsCaps.Set("canSeek", Napi::Boolean::New(env, update->capabilities.canSeek));
		jsUpdate.Set("capabilities", jsCaps);

		// Metadata
		Napi::Object jsMetadata = Napi::Object::New(env);
		if(update->metadata.has_value()){
			jsMetadata.Set("id", Napi::String::New(env, std::u16string(update->metadata->id.begin(), update->metadata->id.end())));
			jsMetadata.Set("title", Napi::String::New(env, std::u16string(update->metadata->title.begin(), update->metadata->title.end())));
			jsMetadata.Set("artist", Napi::String::New(env, std::u16string(update->metadata->artist.begin(), update->metadata->artist.end())));
			jsMetadata.Set("album", Napi::String::New(env, std::u16string(update->metadata->album.begin(), update->metadata->album.end())));
			jsMetadata.Set("albumArtist", Napi::String::New(env, std::u16string(update->metadata->albumArtist.begin(), update->metadata->albumArtist.end())));
			jsMetadata.Set("length", Napi::Number::New(env, update->metadata->length));

			Napi::Array jsArtists = Napi::Array::New(env, update->metadata->artists.size()), jsAlbumArtists = Napi::Array::New(env, update->metadata->albumArtists.size());
			for(int i = 0; i < update->metadata->artists.size(); i++)
				jsArtists.Set(i, Napi::String::New(env, std::u16string(update->metadata->artists[i].begin(), update->metadata->artists[i].end())));
			for(int i = 0; i < update->metadata->albumArtists.size(); i++)
				jsAlbumArtists.Set(i, Napi::String::New(env, std::u16string(update->metadata->albumArtists[i].begin(), update->metadata->albumArtists[i].end())));
			jsMetadata.Set("artists", jsArtists);
			jsMetadata.Set("albumArtists", jsAlbumArtists);

			Napi::Object jsArtData = Napi::Object::New(env);
			jsArtData.Set("size", Napi::Number::New(env, update->metadata->artData.size));
			jsArtData.Set("type", Napi::String::New(env, std::u16string(update->metadata->artData.type.begin(), update->metadata->artData.type.end())));
			jsArtData.Set("data", Napi::Buffer<uint8_t>::New(env, update->metadata->artData.data, update->metadata->artData.size));
			jsMetadata.Set("artData", jsArtData);
		}

		jsUpdate.Set("metadata", jsMetadata);
		return jsUpdate;
	}

	return env.Undefined();
}

Napi::Value WrappedPlayer::Play(const Napi::CallbackInfo& info){
	this->_player.Play();
	return info.Env().Undefined();
}

Napi::Value WrappedPlayer::Pause(const Napi::CallbackInfo& info){
	this->_player.Pause();
	return info.Env().Undefined();
}

Napi::Value WrappedPlayer::PlayPause(const Napi::CallbackInfo& info){
	this->_player.PlayPause();
	return info.Env().Undefined();
}

Napi::Value WrappedPlayer::Stop(const Napi::CallbackInfo& info){
	this->_player.Stop();
	return info.Env().Undefined();
}

Napi::Value WrappedPlayer::Next(const Napi::CallbackInfo& info){
	this->_player.Next();
	return info.Env().Undefined();
}

Napi::Value WrappedPlayer::Previous(const Napi::CallbackInfo& info){
	this->_player.Previous();
	return info.Env().Undefined();
}

Napi::Value WrappedPlayer::Shuffle(const Napi::CallbackInfo& info){
	this->_player.Shuffle();
	return info.Env().Undefined();
}

Napi::Value WrappedPlayer::Repeat(const Napi::CallbackInfo& info){
	this->_player.Repeat();
	return info.Env().Undefined();
}

Napi::Value WrappedPlayer::Seek(const Napi::CallbackInfo& info){
	float offset = info[0].As<Napi::Number>().FloatValue();
	this->_player.Seek(offset);
	return info.Env().Undefined();
}

Napi::Value WrappedPlayer::SeekPercentage(const Napi::CallbackInfo& info){
	float percentage = info[0].As<Napi::Number>().FloatValue();
	this->_player.SeekPercentage(percentage);
	return info.Env().Undefined();
}

Napi::Value WrappedPlayer::GetPosition(const Napi::CallbackInfo& info){
	float position = this->_player.GetPosition();
	return Napi::Number::New(info.Env(), position);
}

Napi::Value WrappedPlayer::SetPosition(const Napi::CallbackInfo& info){
	float position = info[0].As<Napi::Number>().FloatValue();
	this->_player.SetPosition(position);
	return info.Env().Undefined();
}

Napi::Value WrappedPlayer::SetVolume(const Napi::CallbackInfo& info){
	float volume = info[0].As<Napi::Number>().FloatValue();
	this->_player.SetVolume(volume);
	return info.Env().Undefined();
}