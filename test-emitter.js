const winPlayer = require("./emitter").default;

async function main() {
	const playerManager = await winPlayer();
	if (playerManager) {
		playerManager.on("SystemSessionChanged", (aumid) => {
			console.log("manager event: SystemSessionChanged");
			console.log("system session:", aumid);
		});

		playerManager.on("ActiveSessionChanged", (aumid) => {
			console.log("manager event: ActiveSessionChanged");
			console.log("active session:", aumid);
		});

		playerManager.on("SessionsChanged", (keys) => {
			console.log("manager event: SessionsChanged");
			console.log("tracked sessions:", keys);
		});

		playerManager.on("MediaPropertiesChanged", (status) => {
			console.log("player event: MediaPropertiesChanged");
			console.log("status:", status);
		});

		playerManager.on("PlaybackInfoChanged", (status) => {
			console.log("player event: PlaybackInfoChanged");
			console.log("status:", status);
		});

		playerManager.on("TimelinePropertiesChanged", (position) => {
			console.log("player event: TimelinePropertiesChanged");
			console.log("position:", position);
		});
	} else {
		console.error("whoops try again");
		process.exit(1);
	}
}

main();