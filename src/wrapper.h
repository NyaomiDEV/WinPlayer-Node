#ifndef WINPLAYER_NODE_WRAPPER_H
#define WINPLAYER_NODE_WRAPPER_H

#include <napi.h>
#include "lib/winPlayer.h"

class WrappedPlayer : public Napi::ObjectWrap<WrappedPlayer> {
    public:
        WrappedPlayer(const Napi::CallbackInfo& info);
        static Napi::Object Init(Napi::Env env, Napi::Object exports);
    private:
        Player _player;
		Napi::Value getUpdate(const Napi::CallbackInfo& info);
		Napi::Value Play(const Napi::CallbackInfo& info);
		Napi::Value Pause(const Napi::CallbackInfo& info);
		Napi::Value PlayPause(const Napi::CallbackInfo& info);
		Napi::Value Stop(const Napi::CallbackInfo& info);
		Napi::Value Next(const Napi::CallbackInfo& info);
		Napi::Value Previous(const Napi::CallbackInfo& info);
		Napi::Value Shuffle(const Napi::CallbackInfo& info);
		Napi::Value Repeat(const Napi::CallbackInfo& info);
		Napi::Value Seek(const Napi::CallbackInfo& info);
		Napi::Value SeekPercentage(const Napi::CallbackInfo& info);
		Napi::Value GetPosition(const Napi::CallbackInfo& info);
		Napi::Value SetPosition(const Napi::CallbackInfo& info);
		Napi::Value GetVolume(const Napi::CallbackInfo& info);
		Napi::Value SetVolume(const Napi::CallbackInfo& info);
};

#endif // WINPLAYER_NODE_WRAPPER_H
