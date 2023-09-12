# WinPlayer-Node

Now featuring: Rust rewrite!

HUGE thanks to Viola for helping me learn Rust! <3

## Quickstart

```ts
import winplayer, { Status, Position } from "winplayer-rs/emitter";

// in an async body
const playerManager = await winplayer();
if (playerManager) {
	playerManager.on("MediaPropertiesChanged", (status: Status) => {
		console.log(status);
	});

	playerManager.on("PlaybackInfoChanged", (status: Status) => {
		console.log(status);
	});

	playerManager.on("TimelinePropertiesChanged", (position: Position) => {
		console.log(position);
	});
}
```

Also please look at [test.js](test.js) to use the native bindings, or [test-emitter.js](test-emitter.js) for a comprehensive example of the events emitted by the emitter wrapper.

It is highly encouraged to use the emitter wrapper if you're in a rush, otherwise please use the bindings directly and implement your own stuff that way.