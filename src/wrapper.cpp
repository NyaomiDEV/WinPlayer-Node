#include "wrapper.h"
#include "updateworker.h"

#include <chrono>

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
	auto promise = Napi::Promise::Deferred::New(env);
	UpdateWorker *wk = new UpdateWorker(this->_player, promise);
	wk->Queue();
	return promise.Promise();
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
	Napi::Object jsPosition = Napi::Object::New(info.Env());
	jsPosition.Set("howMuch", Napi::Number::New(info.Env(), position));
	jsPosition.Set("when",
		Napi::Date::New(
			info.Env(),
			std::chrono::duration_cast<std::chrono::milliseconds>(
				std::chrono::system_clock::now().time_since_epoch()
			).count()
		)
	);
	return jsPosition;
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