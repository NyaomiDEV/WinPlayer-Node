#include <napi.h>
#include "wrapper.cpp"

Napi::Object Init(Napi::Env env, Napi::Object exports){
	WrappedPlayer::Init(env, exports);
	return exports;
}

NODE_API_MODULE(NODE_GYP_MODULE_NAME, Init);