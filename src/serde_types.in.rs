use chrono::{DateTime, UTC};
use linked_hash_map::LinkedHashMap;
use std::option::Option;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    #[serde(rename="recordingid")]
    recording_id: Uuid,
    #[serde(rename="releaseid")]
    release_id: Option<Uuid>,
    #[serde(rename="releasetrackid")]
    release_track_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Playlist {
    name: String,
    description: Option<String>,
    tracklist: LinkedHashMap<Uuid, Track>,
    timestamp: DateTime<UTC>,
    uuid: Uuid,

}
