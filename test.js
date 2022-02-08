const Player = require("./");

/** @type {import("./").IPlayer} */
let player;

async function onUpdate(){
    const update = await player.getUpdate();
    console.log(require("util").inspect(update, false, null, true /* enable colors */));
    await require("fs/promises").writeFile("test.png", update.metadata.artData?.data);
}

player = new Player(onUpdate);


/* (async()=>{
    while(1) await new Promise(r => setTimeout(()=>{
        console.log(player.GetPosition());
        r();
    }, 1 * 1000));
})(); */
