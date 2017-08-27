extern crate chrono;
extern crate edabo;
extern crate serde_json;
extern crate uuid;

use edabo::types::{Playlist, Track};
use std::collections::HashSet;

fn get_playlist() -> Playlist {
    let track = Track {
        release_track_id: Some(uuid::Uuid::parse_str("d71b7b2d-075c-3c09-8a3f-d050b121f3ab").unwrap()),
        release_id: Some(uuid::Uuid::parse_str("b6f23b8f-1b0f-4167-92e8-d276164e1019").unwrap()),
        recording_id: uuid::Uuid::parse_str("fefd550f-b68e-4c11-b4d6-dfb4836a820e").unwrap()
    };

    let mut tracklist = HashSet::new();
    tracklist.insert(track);

    Playlist::new("foo".to_string(),
                  None,
                  tracklist)
}

#[test]
fn playlist_roundtrip() {
    let before = get_playlist();
    let serialized = serde_json::to_string(&before).unwrap();
    let after = serde_json::from_str(serialized.as_str()).unwrap();
    assert_eq!(before, after)
}

#[test]
fn playlist_from_file() {
    let mut expected = get_playlist();
    expected.uuid = uuid::Uuid::parse_str("69149c42-dd5d-46c2-83bc-0fc801d35dbc").unwrap();
    let read = Playlist::from_file("tests/data/playlist.edabo");
    let actual = read.unwrap();
    assert_eq!(expected, actual)
}

#[test]
fn playlist_from_str() {
    let before = get_playlist();
    let serialized = serde_json::to_string(&before).unwrap();
    let after = Playlist::from_str(serialized.as_str());
    let actual = after.unwrap();
    assert_eq!(before, actual)
}
