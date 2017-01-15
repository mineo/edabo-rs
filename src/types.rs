use clap::App;
use mpd::Song;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error as SerdeDeError, MapVisitor, Visitor};
use std::error::Error;
use std::str;

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
                        where E: SerdeError {
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

/// A trait for the subcommands used by the CLI entry point.
pub trait Command {
    /// The name to use for this command on the command line.
    fn name(&self) -> &str;

    /// Build a new `App` that parses this subcommand.
    fn build_subcommand<'a, 'b>(&self) -> App<'a, 'b>;

    /// Perform the action of this subcommand.
    fn run(&self) -> ();
}
