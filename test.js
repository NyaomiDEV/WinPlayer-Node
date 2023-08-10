const Player = require(".");

/** @type {import(".").Player} */
let player;

async function onUpdate(){
    const update = await player.getUpdate();
    console.log(update);
}

player = new Player(onUpdate);

setInterval(() => console.log(player.GetPosition()), 10);
