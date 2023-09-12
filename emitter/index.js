"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.WinPlayer = void 0;
const __1 = require("..");
const events_1 = require("events");
class WinPlayer extends events_1.EventEmitter {
    playerManager;
    denylist;
    player = null;
    constructor(playerManager, denylist) {
        super();
        this.playerManager = playerManager;
        this.denylist = denylist ?? [];
        const _managerEvents = async () => {
            for (;;) {
                if (!this.playerManager)
                    break;
                const evt = await this.playerManager.pollNextEvent();
                switch (evt) {
                    case "ActiveSessionChanged":
                        this.player = null;
                        const player = this.playerManager.getActiveSession();
                        if (player) {
                            this.player = player;
                            _playerEvents();
                        }
                        break;
                    case "SystemSessionChanged":
                        this.playerManager.updateSystemSession();
                        break;
                    case "SessionsChanged":
                        this.playerManager.updateSessions(this.denylist);
                        break;
                }
                this.emit(evt);
            }
        };
        const _playerEvents = async () => {
            for (;;) {
                if (!this.player)
                    break;
                const evt = await this.player.pollNextEvent();
                switch (evt) {
                    case "PlaybackInfoChanged":
                        this.emit(evt, await this.player.getStatus());
                        break;
                    case "TimelinePropertiesChanged":
                        this.emit(evt, await this.player.getPosition(false));
                        break;
                    case "MediaPropertiesChanged":
                        this.emit(evt, await this.player.getStatus());
                        break;
                }
            }
        };
        _managerEvents();
    }
    async getFriendlyName() {
        if (this.player)
            return await (0, __1.getFriendlyNameFor)(await this.player.getAumid());
        return null;
    }
    async getStatus() {
        return this.player?.getStatus();
    }
    async Play() {
        return await this.player?.play();
    }
    async Pause() {
        return await this.player?.pause();
    }
    async PlayPause() {
        return await this.player?.playPause();
    }
    async Stop() {
        return await this.player?.stop();
    }
    async Next() {
        return await this.player?.next();
    }
    async Previous() {
        return await this.player?.previous();
    }
    async Shuffle() {
        const shuffle = await this.player?.getShuffle();
        return this.player?.setShuffle(!shuffle);
    }
    async Repeat() {
        const repeat = await this.player?.getRepeat();
        switch (repeat) {
            case "List":
            default:
                return await this.player?.setRepeat("None");
            case "None":
                return await this.player?.setRepeat("Track");
            case "Track":
                return await this.player?.setRepeat("List");
        }
    }
    async Seek(offset) {
        return await this.player?.seek(offset);
    }
    async SeekPercentage(percentage) {
        return await this.player?.seekPercentage(percentage);
    }
    async SetPosition(position) {
        return await this.player?.setPosition(position);
    }
    async GetPosition() {
        const pos = await this.player?.getPosition(true);
        if (!pos) {
            return {
                howMuch: 0,
                when: new Date(0)
            };
        }
        return pos;
    }
}
exports.WinPlayer = WinPlayer;
async function init() {
    const playerManager = await (0, __1.getPlayerManager)();
    if (playerManager)
        return new WinPlayer(playerManager);
    return undefined;
}
exports.default = init;
