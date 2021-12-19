# WinPlayer-Node

## The aim

Controlling media playback programmatically is an art. Doing so on Windows is also a problem. This project aims to be able to query and control media playback on Windows using the native Windows Runtime.

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

