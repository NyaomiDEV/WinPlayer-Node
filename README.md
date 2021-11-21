# WinPlayer-Node

## The aim

Controlling media playback programmatically is an art. Doing so on Windows is also a problem. This project aims to be able to query and control media playback on Windows using the native Windows Runtime.

## Current problems
- Cover art retrieval is a hit and miss (and mostly a miss) and it causes either segfaults or v8 crashes (depends on Node version, it seems)
- Because cover art retrieval is a hit and miss, Electron apps flat out crash upon startup.
- getPosition() doesn't work reliably. It needs some thought to make it report the almost accurate position at the time of the method call (right now it reports position as it was last updated by the player itself)

## To do
- All asynchronous operations in WinRT should be mapped to promises and be resolved asynchronously
- Fix the damn cover art crashing the shit out of this library

## Usage

```js
	import Player from "winplayer-node";

	let player;

	function onUpdate(){
		const update = player.getUpdate();
		console.log(update);
	}

	player = new Player(onUpdate);
```

Consult the [type definitions file](index.d.ts) for all the available methods and return types.

## Why not NodeRT?

It is old, it seems unsupported and it does not support VS2022 and Windows 11 SDK. I know I could've forked it but the hard dependency on `nan` is a problem for Electron. I wanted to make this on `napi` to address some of those issues, plus keeping a level of forward-compatibility.

