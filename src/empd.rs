use mpd::Client;
use std::convert::From;
use types::*;

pub fn connect() -> Result<Client, EdaboError> {
    Client::connect("127.0.0.1:6600").map_err(From::from)
}

pub fn current_playlist() -> Result<Playlist, EdaboError> {
    connect().
        and_then(|mut conn| conn.queue().map_err(From::from)).
        and_then(|queue| queue.iter().map(|song| Track::from_song(song)).collect()).
        and_then(|tracks|
                 Ok(Playlist::new("Current".to_string(),
                                  Some("The current playlist".to_string()),
                                  tracks)))
}

pub fn current_track() -> Result<Track, EdaboError> {
    connect().
        and_then(|mut conn| conn.currentsong().map_err(From::from)).
        and_then(|optsong| optsong.
                 ok_or_else(|| EdaboError{ kind: ErrorKind::NoCurrentSong,
                                           detail: None}).
                 and_then(|s| Track::from_song(&s)))
}
