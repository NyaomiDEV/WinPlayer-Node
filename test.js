const Player = require("./");

/** @type {import("./").IPlayer} */
let player;
function onUpdate(){
    const update = player.getUpdate();
    console.log(require("util").inspect(update, false, null, true /* enable colors */));
}

player = new Player(onUpdate);


(async()=>{
    while(1) await new Promise(r => setTimeout(()=>{
        console.log(player.GetPosition());
        r();
    }, 1 * 1000));
})();
