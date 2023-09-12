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
    Play(): Promise<boolean | undefined>;
    Pause(): Promise<boolean | undefined>;
    PlayPause(): Promise<boolean | undefined>;
    Stop(): Promise<boolean | undefined>;
    Next(): Promise<boolean | undefined>;
    Previous(): Promise<boolean | undefined>;
    Shuffle(): Promise<boolean | undefined>;
    Repeat(): Promise<boolean | undefined>;
    Seek(offset: number): Promise<boolean | undefined>;
    SeekPercentage(percentage: number): Promise<boolean | undefined>;
    SetPosition(position: number): Promise<boolean | undefined>;
    GetPosition(): Promise<import("..").Position>;
}
export default function init(): Promise<WinPlayer | undefined>;
