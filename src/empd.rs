use mpd::Client;
use std::convert::From;
use types::*;

pub fn connect() -> Result<Client, EdaboError> {
    Client::connect("127.0.0.1:6600").map_err(|e| From::from(e))
}
