use mpd::Client;
use std::convert::From;
use types::*;

pub fn connect() -> Result<Client, EdaboError> {
    Client::connect("127.0.0.1:6600").map_err(|e| From::from(e))
}

pub fn current_playlist() -> Result<Playlist, EdaboError> {
    connect().
        and_then(|mut conn| conn.queue().map_err(|e| From::from(e))).
        and_then(|queue| queue.iter().map(|song| Track::from_song(song)).collect()).
        and_then(|tracks|
                 Ok(Playlist::new("Current".to_string(),
                                  Some("The current playlist".to_string()),
                                  tracks)))
}
