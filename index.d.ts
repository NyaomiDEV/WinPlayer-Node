declare const PlayerImpl: Player;
export default PlayerImpl;

export declare type Player = {
	constructor(callback: Function): Player;
	getUpdate(): Promise<Update | null>;
	Play(): void;
	Pause(): void;
	PlayPause(): void;
	Stop(): void;
	Next(): void;
	Previous(): void;
	Shuffle(): void;
	Repeat(): void;
	Seek(offset: number): void;
	SeekPercentage(percentage: number): void;
	GetPosition(): Position;
	SetPosition(position: number): void;
	// @deprecated
	GetVolume(): number;
	// @deprecated
	SetVolume(volume: number): number;
};

export declare type Position = {
	howMuch: number;
	when: Date;
};

export declare type ArtData = {
	data: Buffer;
	type: string[];
};

export declare type Metadata = {
	id: string;
	title: string;
	artist: string;
	artists: string[];
	album: string;
	albumArtist: string;
	albumArtists: string[];
	artData: ArtData;
	length: number;
};

export declare type Capabilities = {
	canControl: boolean;
	canPlayPause: boolean;
	canGoNext: boolean;
	canGoPrevious: boolean;
	canSeek: boolean;
};

export declare type Update = {
	provider: "WinPlayer";
	metadata: Metadata;
	capabilities: Capabilities;
	status: string;
	loop: string;
	shuffle: boolean;
	volume: number;
	elapsed: Position;
	app: string;
	appName: string;
};
