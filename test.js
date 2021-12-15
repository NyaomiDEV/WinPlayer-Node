const Player = require("./");

/** @type {import("./").IPlayer} */
let player;
function onUpdate(){
    const update = player.getUpdate();
    console.log(update);
    if (update.metadata.artData) {
        require("fs").writeFileSync("test.png", update.metadata.artData.data);
    }
}

player = new Player(onUpdate);


(async()=>{
    while(1) await new Promise(r => setTimeout(()=>{
        //console.log(player.GetPosition());
        r();
    }, 1 * 1000));
})();
