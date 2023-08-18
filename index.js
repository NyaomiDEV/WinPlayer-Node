const { existsSync } = require("fs");

if(existsSync("./build/Debug/winplayerbinding.node"))
    module.exports = require("./build/Debug/winplayerbinding.node").Player;
else
    module.exports = require("./build/Release/winplayerbinding.node").Player;
