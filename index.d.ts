/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export class ExternalObject<T> {
  readonly '': {
    readonly '': unique symbol
    [K: symbol]: T
  }
}
export interface ArtData {
  data: Buffer
  mimetype: string
}
export interface Metadata {
  album?: string
  albumArtist?: string
  albumArtists?: Array<string>
  artist: string
  artists: Array<string>
  artData?: ArtData
  id?: string
  length: number
  title: string
}
export interface Capabilities {
  canControl: boolean
  canPlayPause: boolean
  canGoNext: boolean
  canGoPrevious: boolean
  canSeek: boolean
}
export interface Position {
  howMuch: number
  when: Date
}
export interface Status {
  metadata?: Metadata
  capabilities: Capabilities
  status: string
  isLoop: string
  shuffle: boolean
  volume: number
  elapsed?: Position
  app?: string
}
export function getPlayerManager(): Promise<PlayerManager | null>
export function getFriendlyNameFor(aumid: string): Promise<string | null>
export type JsPlayer = Player
export class Player {
  constructor(player: ExternalObject<Player>)
  pollNextEvent(): Promise<string>
  getStatus(): Promise<Status>
  getAumid(): Promise<string>
  play(): Promise<boolean>
  pause(): Promise<boolean>
  playPause(): Promise<boolean>
  stop(): Promise<boolean>
  getPlaybackStatus(): Promise<string>
  next(): Promise<boolean>
  previous(): Promise<boolean>
  setShuffle(value: boolean): Promise<boolean>
  getShuffle(): Promise<boolean>
  setRepeat(value: string): Promise<boolean>
  getRepeat(): Promise<string>
  seek(offsetS: number): Promise<boolean>
  seekPercentage(percentage: number): Promise<boolean>
  setPosition(positionS: number): Promise<boolean>
  getPosition(wantsCurrentPosition: boolean): Promise<Position | null>
}
export type JsPlayerManager = PlayerManager
export class PlayerManager {
  constructor(playerManager: ExternalObject<PlayerManager>)
  pollNextEvent(): Promise<string>
  getActiveSession(): Player | null
  getSession(aumid: string): Player | null
  getSessionsKeys(): Array<string>
  getSystemSession(): Player | null
  updateSystemSession(): void
  updateSessions(denylist?: Array<string> | undefined | null): void
}
