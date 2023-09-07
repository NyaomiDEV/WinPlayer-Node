const winPlayer = require(".");

async function main() {
	const playerManager = await winPlayer.getPlayerManager();
	let player;

	async function pmEventPolling() {
		while (evt = await playerManager.pollNextEvent()) {
			console.log("manager event", evt);
			switch (evt) {
				case "SystemSessionChanged":
					playerManager.updateSystemSession();
					break;
				case "ActiveSessionChanged":
					player = await playerManager.getActiveSession();
					if (!player)
						console.log("manager: WTF?!");
					else {
						console.log("player attached:", await player?.getAumid());
						eventPolling();
					}
					break;
				case "SessionsChanged":
					playerManager.updateSessions(null);
					break;
			}
		}
	}

	async function eventPolling() {
		while (evt = await player?.pollNextEvent()) {
			console.log("player event", evt);
			switch (evt) {
				case "PlaybackInfoChanged":
					console.log("status", await player?.getStatus())
					break;
				case "TimelinePropertiesChanged":
					console.log("status and pos", await player?.getStatus(), await player?.getPosition(false))
					break;
			}
		}
	}

	pmEventPolling()
}

main();