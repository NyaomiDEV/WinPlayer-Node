const winplayernode = require("./build/Release/winplayerbinding.node");

console.log("test");
console.log(winplayernode);

(async()=>{
    while(true) await new Promise(r => setTimeout(r, 1000))
})()