use chrono::{DateTime, Utc};
use clap::{App, ArgMatches};
use commands::get_playlist_dir;
use mpd::Song;
use mpd::error::Error as MPDError;
use serde_json;
use std::collections::HashSet;
use std::convert::From;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Error as IOError;
use std::option::Option;
use std::path::Path;
use std::result;
use std::str;
use uuid::{ParseError, Uuid};
use xdg::BaseDirectoriesError;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct Track {
    #[serde(rename = "recordingid")]
    pub recording_id: Uuid,
    #[serde(rename = "releaseid")]
    pub release_id: Option<Uuid>,
    #[serde(rename = "releasetrackid")]
    pub release_track_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Eq)]
pub struct Playlist {
    pub name: String,
    pub description: Option<String>,
    pub tracklist: HashSet<Track>,
    pub timestamp: DateTime<Utc>,
    pub uuid: Uuid,
}
impl Track {
    pub fn from_song(song: &Song) -> result::Result<Track, EdaboError> {
        let ref tags = song.tags;
        let recording_id = tags.get("MUSICBRAINZ_TRACKID")
            .ok_or_else(|| {
                EdaboError {
                    kind: ErrorKind::MissingTagError(
                        song.file.clone(),
                        String::from("recordingid"),
                    ),
                    detail: None,
                }
            })
            .and_then(|value| {
                Uuid::parse_str(value).map_err(|e| {
                    EdaboError {
                        kind: ErrorKind::UuidError(song.file.clone(), e),
                        detail: None,
                    }
                })
            });

        // The release id and release track id are optional, but if the tags
        // exist, they should contain uuids
        let release_id: Result<Option<Uuid>> = match tags.get("MUSICBRAINZ_ALBUMID") {
            None => Ok(None),
            Some(value) => {
                Uuid::parse_str(value)
                    .map_err(|e| {
                        EdaboError {
                            kind: ErrorKind::UuidError(song.file.clone(), e),
                            detail: None,
                        }
                    })
                    .and_then(|v| Ok(Some(v)))
            }
        };

        let release_track_id = match tags.get("MUSICBRAINZ_RELEASETRACKID") {
            None => Ok(None),
            Some(value) => {
                Uuid::parse_str(value)
                    .map_err(|e| {
                        EdaboError {
                            kind: ErrorKind::UuidError(song.file.clone(), e),
                            detail: None,
                        }
                    })
                    .and_then(|v| Ok(Some(v)))
            }
        };

        // TODO: Use collect over Vec<Result<Uuid, EdaboError>> or something
        // similar here
        recording_id.and_then(|id| {
            release_id.and_then(|relid| {
                release_track_id.and_then(|rtid| {
                    Ok(Track {
                        recording_id: id,
                        release_id: relid,
                        release_track_id: rtid,
                    })
                })
            })
        })
    }
}

impl Playlist {
    pub fn from_file<P>(path: P) -> result::Result<Playlist, EdaboError>
    where
        P: AsRef<Path>,
    {
        File::open(path).map_err(From::from).and_then(|file| {
            serde_json::from_reader(file).map_err(From::from)
        })
    }

    pub fn from_name(name: &str) -> result::Result<Playlist, EdaboError> {
        get_playlist_dir().and_then(|mut path| {
            path.push(name);
            path.set_extension("edabo");
            Playlist::from_file(path)
        })
    }

    pub fn from_str(s: &str) -> result::Result<Playlist, EdaboError> {
        serde_json::from_str(s).map_err(From::from)
    }

    pub fn to_file(self: &Self) -> Option<EdaboError> {
        get_playlist_dir()
            .and_then(|mut path| {
                path.push(&self.name);
                path.set_extension("edabo");
                File::create(path).map_err(From::from)
            })
            .and_then(|mut file| {
                serde_json::to_writer_pretty(&mut file, self).map_err(From::from)
            })
            .err()
    }

    pub fn new(name: String, description: Option<String>, tracklist: HashSet<Track>) -> Playlist {
        let uuid = Uuid::new_v4();
        let timestamp = Utc::now();
        Playlist {
            name: name,
            description: description,
            tracklist: tracklist,
            uuid: uuid,
            timestamp: timestamp,
        }
    }
}

impl PartialEq for Playlist {
    fn eq(self: &Playlist, other: &Playlist) -> bool {
        self.name == other.name && self.description == other.description &&
            self.tracklist == other.tracklist && self.uuid == other.uuid
    }
}

/// A trait for the subcommands used by the CLI entry point.
pub trait Command {
    /// The name to use for this command on the command line.
    fn name(&self) -> &str;

    /// Build a new `App` that parses this subcommand.
    fn build_subcommand<'a, 'b>(&self) -> App<'a, 'b>;

    /// Perform the action of this subcommand.
    fn run(&self, &ArgMatches) -> result::Result<(), EdaboError>;
}

#[derive(Debug)]
pub enum ErrorKind {
    ArgumentError,
    IoError(IOError),
    JsonError(serde_json::Error),
    MissingSongError(Uuid),
    MissingTagError(String, String),
    MpdError(MPDError),
    NoCurrentSong,
    UuidError(String, ParseError),
    XdgError(BaseDirectoriesError),
}

#[derive(Debug)]
pub struct EdaboError {
    pub kind: ErrorKind,
    pub detail: Option<String>,
}

impl Error for EdaboError {
    fn description(&self) -> &str {
        "Something went wrong. In the future, I'll even tell you what!"
    }
}

impl fmt::Display for EdaboError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl From<IOError> for EdaboError {
    fn from(e: IOError) -> EdaboError {
        EdaboError {
            kind: ErrorKind::IoError(e),
            detail: None,
        }
    }
}

impl From<MPDError> for EdaboError {
    fn from(e: MPDError) -> EdaboError {
        EdaboError {
            kind: ErrorKind::MpdError(e),
            detail: None,
        }
    }
}

impl From<serde_json::Error> for EdaboError {
    fn from(e: serde_json::Error) -> EdaboError {
        EdaboError {
            kind: ErrorKind::JsonError(e),
            detail: None,
        }
    }
}

impl From<BaseDirectoriesError> for EdaboError {
    fn from(e: BaseDirectoriesError) -> EdaboError {
        EdaboError {
            kind: ErrorKind::XdgError(e),
            detail: None,
        }
    }
}

pub type Result<T> = result::Result<T, EdaboError>;
