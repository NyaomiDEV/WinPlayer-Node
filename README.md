# WinPlayer-Node

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

## The aim

I wanted to make a Windows media controller, so that I can integrate Node apps with it.

## Why not NodeRT?

It is old, it seems unsupported and it does not support VS2022 and Windows 11 SDK. I know I could've forked it but the hard dependency on `nan` is a problem for Electron. I wanted to make this on `napi` to address some of those issues, plus keeping a level of forward-compatibility.

