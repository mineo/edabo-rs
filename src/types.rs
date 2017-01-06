use clap::App;
use mpd::Song;
use std::error::Error;

include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

impl Track {
    pub fn from_song(song: &Song) -> Result<Track, String> {
        let ref tags = song.tags;
        let recording_id = if let Some(value) = tags.get("MUSICBRAINZ_TRACKID") {
            match Uuid::parse_str(value) {
                Err(err) => Err(err.description().to_string()),
                Ok(value) => Ok(value),
            }
        } else {
            Err("No recordingid tag".to_string())
        };

        // TODO: For now, just ignore parse failures
        let release_id = if let Some(value) = tags.get("MUSICBRAINZ_ALBUMID") {
            match Uuid::parse_str(value) {
                Err(_) => None,
                Ok(value) => Some(value),
            }
        } else {
            None
        };

        let release_track_id = if let Some(value) = tags.get("MUSICBRAINZ_RELEASETRACKID") {
            match Uuid::parse_str(value) {
                Err(_) => None,
                Ok(value) => Some(value),
            }
        } else {
            None
        };

        match recording_id {
            Ok(id) => {
                Ok(Track {
                    recording_id: id,
                    release_id: release_id,
                    release_track_id: release_track_id,
                })
            }
            // The LHS Err(reason) has a different type than the RHS one.
            Err(reason) => Err(reason),
        }
    }
}

impl Playlist {
    pub fn new(name: String, description: Option<String>, tracklist: Vec<Track>) -> Playlist {
        let uuid = Uuid::new_v4();
        let timestamp = UTC::now();
        Playlist {
            name: name,
            description: description,
            tracklist: tracklist,
            uuid: uuid,
            timestamp: timestamp,
        }
    }
}

/// A trait for the subcommands used by the CLI entry point.
pub trait Command {
    /// The name to use for this command on the command line.
    fn name(&self) -> &str;

    /// Build a new `App` that parses this subcommand.
    fn build_subcommand<'a, 'b>(&self) -> App<'a, 'b>;

    /// Perform the action of this subcommand.
    fn run(&self) -> ();
}
