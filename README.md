# WinPlayer-Node-rs

## Da fare
- [ ] Porting di tutte le funzioni a Rust
  - [x] addPlayer(std::string const AUMID, GSMTCSession player) -> Non necessario per questo pattern
  - [x] removePlayer(std::string const AUMID) -> Non necessario per questo pattern
  - [x] updatePlayers() -> update_active_session()
  - [/] Player() -> pub new() + pub run()
  - [x] getPlayerName(GSMTCSession player) -> get_session_player_name(session: GSMTCSession)
  - [ ] registerPlayerEvents(std::string const AUMID, GSMTCSession player) -> register_session_events(session: GSMTCSession)
  - [x] getMetadata(GSMTCSession player) -> get_session_metadata(session: GSMTCSession)
  - [x] getCapabilities(GSMTCSession player) -> get_session_capabilities(session: GSMTCSession)
  - [x] getUpdate() -> get_session_status(session: GSMTCSession) + pub get_active_session_status()
  - [ ] public setCallback() -> Forse è necessario per mandare robe al JS context? Possiamo dispatchare eventi?
  - [x] public Play() -> play()
  - [x] public Pause() -> pause()
  - [x] public PlayPause() -> play_pause()
  - [x] public Stop() -> stop()
  - [x] public Next() -> next()
  - [x] public Previous() -> previous()
  - [/] public Shuffle() -> shuffle()
  - [/] public Repeat() -> repeat()
  - [/] public Seek(int const offsetUs) -> seek(offset_us: i64)
  - [/] public SeekPercentage(float const percentage) -> seek_percentage(percentage: f64)
  - [/] public GetPosition() -> get_position()
  - [/] public SetPosition(float const positionS) -> set_position(position_s: f64)
- [ ] Fixare i bug ovvii
- [ ] Implementare una glue per Node (NEON o Napi-rs?)
- [ ] Scrivere test?