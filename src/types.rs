use clap::{App, ArgMatches};
use std::fmt;
use commands::{get_playlist_dir};
use mpd::error::Error as MPDError;
use mpd::Song;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error as SerdeDeError, MapVisitor, Visitor};
use serde_json;
use serde_json::error::Result as SerdeResult;
use std::convert::From;
use std::error::Error;
use std::fs::File;
use std::io::Error as IOError;
use std::path::Path;
use std::str;
use uuid::ParseError;

include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

impl Track {
    pub fn from_song(song: &Song) -> Result<Track, EdaboError> {
        let ref tags = song.tags;
        let recording_id = if let Some(value) = tags.get("MUSICBRAINZ_TRACKID") {
            match Uuid::parse_str(value) {
                Err(err) => Err(From::from(err)),
                Ok(value) => Ok(value),
            }
        } else {
            Err(EdaboError{
                kind: ErrorKind::MissingTagError(String::from("recordingid")),
                detail: None
            })
        };

        // TODO: For now, just ignore parse failures
        let release_id = if let Some(value) = tags.get("MUSICBRAINZ_ALBUMID") {
            match Uuid::parse_str(value) {
                Err(err) => Err(From::from(err)),
                Ok(value) => Ok(value),
            }
        } else {
            Err(EdaboError{
                kind: ErrorKind::MissingTagError(String::from("albumid")),
                detail: None
            })
        };

        let release_track_id = if let Some(value) = tags.get("MUSICBRAINZ_RELEASETRACKID") {
            match Uuid::parse_str(value) {
                Err(err) => Err(From::from(err)),
                Ok(value) => Ok(value),
            }
        } else {
            Err(EdaboError{
                kind: ErrorKind::MissingTagError(String::from("releasetrackid")),
                detail: None
            })
        };

        match recording_id {
            Ok(id) => {
                Ok(Track {
                    recording_id: id,
                    release_id: release_id.ok(),
                    release_track_id: release_track_id.ok(),
                })
            }
            // The LHS Err(reason) has a different type than the RHS one.
            Err(reason) => Err(reason),
        }
    }
}

impl Playlist {
    pub fn from_file<P>(path: P) -> Result<Playlist, EdaboError>
        where P: AsRef<Path>,
    {
        // let file: File = File::open(path).unwrap();
        // let pl: SerdeResult<Playlist> = serde_json::from_reader(file);
        // pl.map_err(|e| From::from(e))

        match File::open(path) {
            Ok(file) => {
                let des: SerdeResult<Playlist> = serde_json::from_reader(file);
                match des {
                    Ok(pl) => Ok(pl),
                    Err(e) => Err(From::from(e))
                }
            }
            ,
            Err(e) => Err(From::from(e))
        }
    }

    pub fn from_name(name: &str) -> Result<Playlist, EdaboError>
    {
        let mut filepath = get_playlist_dir();
        filepath.push(name);
        filepath.set_extension("edabo");
        Playlist::from_file(filepath)
    }

    pub fn from_str(s: &str) -> Result<Playlist, EdaboError>
    {
        serde_json::from_str(s).map_err(|e| From::from(e))
    }

    pub fn new(name: String, description: Option<String>, mut tracklist: Vec<Track>) -> Playlist {
        let uuid = Uuid::new_v4();
        let timestamp = UTC::now();
        let mut track_hash_map = LinkedHashMap::with_capacity(tracklist.len());
        for track in tracklist.drain(..) {
            track_hash_map.insert(track.recording_id, track);
        }
        Playlist {
            name: name,
            description: description,
            tracklist: track_hash_map,
            uuid: uuid,
            timestamp: timestamp,
        }
    }
}

impl Serialize for Playlist {
    fn serialize<S>(self: &Self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer {
        let tracks: Vec<&Track> = self.tracklist.values().collect();
        let mut state = serializer.serialize_struct("Playlist", 5).unwrap();
        serializer.serialize_struct_elt(&mut state, "name", &self.name).
            and_then(| _ | serializer.serialize_struct_elt(&mut state, "description", &self.description)).
            and_then(| _ | serializer.serialize_struct_elt(&mut state, "timestamp", &self.timestamp)).
            and_then(| _ | serializer.serialize_struct_elt(&mut state, "uuid", &self.uuid)).
            and_then(| _ | serializer.serialize_struct_elt(&mut state, "tracklist", &tracks)).
            and_then(| _ | serializer.serialize_struct_end(state))
    }
}

impl Deserialize for Playlist {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer {
        #[allow(non_camel_case_types)]
        enum Field { name, description, timestamp, uuid, tracklist}

        impl Deserialize for Field {
            fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                where D: Deserializer {
                struct FieldVisitor;

                impl Visitor for FieldVisitor {
                    type Value = Field;
                    fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E>
                        where E: SerdeDeError {
                        match value {
                            "name" => Ok(Field::name),
                            "description" => Ok(Field::description),
                            "timestamp" => Ok(Field::timestamp),
                            "uuid" => Ok(Field::uuid),
                            "tracklist" => Ok(Field::tracklist),
                            _ => Err(E::unknown_field(value))
                        }
                    }

                    fn visit_bytes<E>(&mut self, value: &[u8]) -> Result<Self::Value, E>
                        where E: SerdeDeError {
                        match value {
                            b"description" => Ok(Field::description),
                            b"timestamp" => Ok(Field::timestamp),
                            b"uuid" => Ok(Field::uuid),
                            b"tracklist" => Ok(Field::tracklist),
                            _ => unsafe {
                                Err(E::unknown_field(str::from_utf8_unchecked(value)))
                            }
                        }
                    }
                }

                deserializer.deserialize_struct_field(FieldVisitor)
            }
        }

        struct ValuesVisitor;
        impl Visitor for ValuesVisitor {
            type Value = Playlist;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: MapVisitor {
                let mut name: Option<String> = None;
                let mut description: Option<Option<String>> = None;
                let mut tracklist: Option<Vec<Track>> = None;
                let mut timestamp: Option<DateTime<UTC>> = None;
                let mut uuid: Option<Uuid> = None;

                macro_rules! read_fields {
                    ( $($name: ident),* ) => (
                        while let Some(key) = visitor.visit_key::<Field>().unwrap() {
                            match key {
                                $(
                                    Field::$name => {
                                        if $name.is_some() {
                                            return Err(V::Error::duplicate_field("$name"));
                                        }

                                        // TODO: properly track errors here
                                        $name = Some(visitor.visit_value().unwrap());
                                    }
                                )*
                            }
                        }
                    );
                }

                read_fields![name,description,tracklist,timestamp,uuid];

                visitor.end().unwrap();

                description = description.or(Some(None));

                match (name, description, tracklist, timestamp, uuid) {
                    (Some(n), Some(d), Some(tr), Some(ti), Some(u)) => {
                        let mut tracks: LinkedHashMap<Uuid,Track> = LinkedHashMap::with_capacity(tr.len());

                        for track in tr {
                            tracks.insert(track.recording_id, track);
                        }

                        Ok(Playlist {
                            name: n,
                            description: d,
                            tracklist: tracks,
                            timestamp: ti,
                            uuid: u
                        })},
                    _ => Err(V::Error::missing_field("I don't know which"))
                }
            }
        }

        const FIELDS: &'static [&'static str] =
            &["name", "description", "tracklist", "uuid", "timestamp"];
        deserializer.deserialize_struct("Playlist", FIELDS, ValuesVisitor)
    }
}

impl PartialEq for Playlist {
    fn eq(self: &Playlist, other: &Playlist) -> bool {
        self.name == other.name &&
            self.description == other.description &&
            self.tracklist == other.tracklist &&
            self.uuid == other.uuid
    }
}

impl Eq for Playlist {
}
/// A trait for the subcommands used by the CLI entry point.
pub trait Command {
    /// The name to use for this command on the command line.
    fn name(&self) -> &str;

    /// Build a new `App` that parses this subcommand.
    fn build_subcommand<'a, 'b>(&self) -> App<'a, 'b>;

    /// Perform the action of this subcommand.
    fn run(&self, ArgMatches) -> ();
}

#[derive(Debug)]
pub enum ErrorKind {
    IoError(IOError),
    JsonError(serde_json::Error),
    MissingTagError(String),
    MpdError(MPDError),
    UuidError(ParseError),
}

#[derive(Debug)]
pub struct EdaboError {
    pub kind: ErrorKind,
    pub detail: Option<String>
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
        EdaboError{
            kind: ErrorKind::IoError(e),
            detail: None}
    }
}

impl From<MPDError> for EdaboError {
    fn from(e: MPDError) -> EdaboError {
        EdaboError{
            kind: ErrorKind::MpdError(e),
            detail: None}
    }
}

impl From<serde_json::Error> for EdaboError {
    fn from(e: serde_json::Error) -> EdaboError {
        EdaboError{
            kind: ErrorKind::JsonError(e),
            detail: None}
    }
}

impl From<ParseError> for EdaboError {
    fn from(e: ParseError) -> EdaboError {
        EdaboError{
            kind: ErrorKind::UuidError(e),
            detail: None}
    }
}
