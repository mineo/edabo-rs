extern crate chrono;
extern crate edabo;
extern crate serde_json;
extern crate uuid;


#[test]
fn playlist_roundtrip() {
    let track = edabo::types::Track {
        release_track_id: Some(uuid::Uuid::parse_str("d71b7b2d-075c-3c09-8a3f-d050b121f3ab").unwrap()),
        release_id: Some(uuid::Uuid::parse_str("b6f23b8f-1b0f-4167-92e8-d276164e1019").unwrap()),
        recording_id: uuid::Uuid::parse_str("fefd550f-b68e-4c11-b4d6-dfb4836a820e").unwrap()
    };

    let before = edabo::types::Playlist::new("Current".to_string(),
                                             None,
                                             vec![track]);
    let serialized = serde_json::to_string(&before).unwrap();
    let after = serde_json::from_str(serialized.as_str()).unwrap();
    assert_eq!(before, after)
}
