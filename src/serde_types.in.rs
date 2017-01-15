use chrono::{DateTime, UTC};
use linked_hash_map::LinkedHashMap;
use std::option::Option;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Track {
    #[serde(rename="recordingid")]
    pub recording_id: Uuid,
    #[serde(rename="releaseid")]
    pub release_id: Option<Uuid>,
    #[serde(rename="releasetrackid")]
    pub release_track_id: Option<Uuid>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Playlist {
    pub name: String,
    pub description: Option<String>,
    pub tracklist: LinkedHashMap<Uuid, Track>,
    pub timestamp: DateTime<UTC>,
    pub uuid: Uuid,

}
