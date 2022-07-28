const Player = require(".");

/** @type {import(".").Player} */
let player;

async function onUpdate(){
    const update = await player.getUpdate();
    console.log(update);
}

player = new Player(onUpdate);
console.log(player.GetPosition());

/* (async()=>{
    while(1) await new Promise(r => setTimeout(()=>{
        console.log(player.GetPosition());
        r();
    }, 1 * 1000));
})(); */
