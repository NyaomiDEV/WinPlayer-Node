#ifndef WINPLAYER_UPDATE_WORKER_H
#define WINPLAYER_UPDATE_WORKER_H

#include <napi.h>
#include "lib/winPlayer.h"

class UpdateWorker : public Napi::AsyncWorker {
	private:
		static Napi::Value noop(Napi::CallbackInfo const& info){
			return info.Env().Undefined();
		}

		Napi::Reference<Napi::Function> const getFakeCallback(Napi::Env const& env){
			Napi::Reference<Napi::Function> cb = Napi::Reference<Napi::Function>::New(Napi::Function::New(env, noop), 1);
			cb.SuppressDestruct();
			return cb;
		}


		Napi::Promise::Deferred promise;
		Player* player;
		std::optional<Update> currentUpdate;
	public:
		UpdateWorker(Player &player, Napi::Promise::Deferred const& promise);
		~UpdateWorker();
		void Execute() override;
		void OnOK() override;
};

#endif // WINPLAYER_UPDATE_WORKER_H
