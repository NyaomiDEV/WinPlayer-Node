const Player = require("./");

/** @type {import("./").IPlayer} */
let player;

function onUpdate(){
    const update = player.getUpdate();
    console.log(update);
    //console.log(Buffer.from(update.metadata.artData.data).toString("base64"));
}

player = new Player(onUpdate);

(async()=>{
    while(1) await new Promise(r => setTimeout(r, 10 * 1000));
})();
