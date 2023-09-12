/// <reference types="node" />
import { Player, PlayerManager } from "..";
import { EventEmitter } from "events";
export type { Status, Position, ArtData, Metadata, Capabilities } from "..";
export declare class WinPlayer extends EventEmitter {
    playerManager: PlayerManager;
    denylist: string[];
    player: Player | null;
    constructor(playerManager: PlayerManager, denylist?: string[]);
    getFriendlyName(): Promise<string | null>;
    getStatus(): Promise<import("..").Status | undefined>;
    play(): Promise<boolean | undefined>;
    pause(): Promise<boolean | undefined>;
    playPause(): Promise<boolean | undefined>;
    stop(): Promise<boolean | undefined>;
    next(): Promise<boolean | undefined>;
    previous(): Promise<boolean | undefined>;
    shuffle(): Promise<boolean | undefined>;
    getShuffle(): Promise<boolean | undefined>;
    setShuffle(value: boolean): Promise<boolean | undefined>;
    repeat(): Promise<boolean | undefined>;
    getRepeat(): Promise<string | undefined>;
    setRepeat(value: string): Promise<boolean | undefined>;
    seek(offset: number): Promise<boolean | undefined>;
    seekPercentage(percentage: number): Promise<boolean | undefined>;
    setPosition(position: number): Promise<boolean | undefined>;
    getPosition(): Promise<import("..").Position>;
}
export default function init(): Promise<WinPlayer | undefined>;
