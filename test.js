const winPlayer = require(".");

async function main() {
	const playerManager = await winPlayer.getPlayerManager();
	let player;

	async function pmEventPolling() {
		while (evt = await playerManager.pollNextEvent()) {
			console.log("manager event", evt);
			switch (evt) {
				case "CurrentSessionChanged":
					playerManager.updateSystemSession();
					break;
				case "SessionsChanged":
					playerManager.updateSessions(null);
					player = await playerManager.getActiveSession();
					if (!player)
						console.log("manager: WTF?!");
					else {
						console.log("player attached:", await player.getAumid());
						eventPolling();
					}
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
					console.log(await player.getStatus(), await player.getPosition(false))
					break;
			}
		}
	}

	pmEventPolling()
}

main();