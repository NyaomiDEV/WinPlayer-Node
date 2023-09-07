const winPlayer = require(".");

const playerManager = await winPlayer.getPlayerManager();
let player;

async function pmEventPolling(){
	while(evt = await playerManager.pollNextEvent()) {
		console.log("manager event", evt);
		switch(evt){
			case "CurrentSessionUpdated":
				playerManager.updateSystemSession();
				break;
			case "SessionsUpdated":
				playerManager.updateSessions(null);
				player = playerManager.getActiveSession();
				if (!player)
					console.log("manager: WTF?!");
				else
					eventPolling();
				break;
		}
	}
}

async function eventPolling() {
	while (evt = await player.pollNextEvent()) {
		console.log("manager event", evt);
		switch (evt) {
			case "PlaybackInfoChanged":
				console.log(await player.getStatus())
				break;
			case "TimelinePropertiesChanged":
				console.log(await player.getPosition(false))
				break;
		}
	}
}

pmEventPolling()
