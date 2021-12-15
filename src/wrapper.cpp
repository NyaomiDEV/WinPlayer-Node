#include "wrapper.h"

Napi::Object WrappedPlayer::Init(Napi::Env env, Napi::Object exports){
	Napi::Function func = DefineClass(env, "Player", {
		InstanceMethod("getUpdate", &WrappedPlayer::getUpdate),
		InstanceMethod("Play", &WrappedPlayer::Play),
		InstanceMethod("Pause", &WrappedPlayer::Pause),
		InstanceMethod("PlayPause", &WrappedPlayer::PlayPause),
		InstanceMethod("Stop", &WrappedPlayer::Stop),
		InstanceMethod("Next", &WrappedPlayer::Next),
		InstanceMethod("Previous", &WrappedPlayer::Previous),
		InstanceMethod("Shuffle", &WrappedPlayer::Shuffle),
		InstanceMethod("Repeat", &WrappedPlayer::Repeat),
		InstanceMethod("Seek", &WrappedPlayer::Seek),
		InstanceMethod("SeekPercentage", &WrappedPlayer::SeekPercentage),
		InstanceMethod("GetPosition", &WrappedPlayer::GetPosition),
		InstanceMethod("SetPosition", &WrappedPlayer::SetPosition),
		InstanceMethod("GetVolume", &WrappedPlayer::GetVolume),
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
	auto update = this->_player.getUpdate().get();
	if(update.has_value()){
		Napi::Object jsUpdate = Napi::Object::New(env);

		jsUpdate.Set("provider", Napi::String::New(env, "WinPlayer"));
		jsUpdate.Set("status", stringOrUndefined(env, update->status));
		jsUpdate.Set("loop", stringOrUndefined(env, update->loop));
		jsUpdate.Set("app", stringOrUndefined(env, update->app));
		jsUpdate.Set("appName", stringOrUndefined(env, update->appName));
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
			jsMetadata.Set("id", stringOrUndefined(env, update->metadata->id));
			jsMetadata.Set("title", stringOrUndefined(env, update->metadata->title));
			jsMetadata.Set("artist", stringOrUndefined(env, update->metadata->artist));
			jsMetadata.Set("album", stringOrUndefined(env, update->metadata->album));
			jsMetadata.Set("albumArtist", stringOrUndefined(env, update->metadata->albumArtist));
			jsMetadata.Set("length", Napi::Number::New(env, update->metadata->length));

			Napi::Array jsArtists = Napi::Array::New(env),
						jsAlbumArtists = Napi::Array::New(env);
			for(int i = 0; i < update->metadata->artists.size(); i++){
				auto value = stringOrUndefined(env, update->metadata->artists[i]);
				if(value != env.Undefined())
					jsArtists.Set(i, value);
			}
			for(int i = 0; i < update->metadata->albumArtists.size(); i++){
				auto value = stringOrUndefined(env, update->metadata->albumArtists[i]);
				if (value != env.Undefined())
					jsAlbumArtists.Set(i, value);
			}
			jsMetadata.Set("artists", jsArtists);
			jsMetadata.Set("albumArtists", jsAlbumArtists);

			if(update->metadata->artData.data.size() > 0){
				Napi::Object jsArtData = Napi::Object::New(env);

				Napi::Array jsArtType = Napi::Array::New(env, update->metadata->artData.type.size());
				for (int i = 0; i < update->metadata->artData.type.size(); i++)
					jsArtType.Set(i, Napi::String::New(env, update->metadata->artData.type[i]));
				jsArtData.Set("type", jsArtType);

				Napi::Buffer buf = Napi::Buffer<uint8_t>::Copy(env, update->metadata->artData.data.data(), update->metadata->artData.data.size());
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
		return jsUpdate;
	}

	return env.Null();
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

Napi::Value WrappedPlayer::GetVolume(const Napi::CallbackInfo& info){
	float volume = this->_player.GetVolume();
	return Napi::Number::New(info.Env(), volume);
}

Napi::Value WrappedPlayer::SetVolume(const Napi::CallbackInfo& info){
	float volume = info[0].As<Napi::Number>().FloatValue();
	this->_player.SetVolume(volume);
	return info.Env().Undefined();
}