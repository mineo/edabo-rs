extern crate edabo;
extern crate uuid;

use edabo::types::Track;
use std::collections::HashSet;

#[test]
fn hashes_equally() {
    let track_complete_metadata = Track {
        release_track_id: Some(
            uuid::Uuid::parse_str("d71b7b2d-075c-3c09-8a3f-d050b121f3ab").unwrap(),
        ),
        release_id: Some(
            uuid::Uuid::parse_str("b6f23b8f-1b0f-4167-92e8-d276164e1019").unwrap(),
        ),
        recording_id: uuid::Uuid::parse_str("fefd550f-b68e-4c11-b4d6-dfb4836a820e").unwrap(),
    };

    let another_track_complete_metadata = Track {
        release_track_id: Some(
            uuid::Uuid::parse_str("d71b7b2d-075c-3c09-8a3f-d050b121f3ab").unwrap(),
        ),
        release_id: Some(
            uuid::Uuid::parse_str("b6f23b8f-1b0f-4167-92e8-d276164e1019").unwrap(),
        ),
        recording_id: uuid::Uuid::parse_str("fefd550f-b68e-4c11-b4d6-dfb4836a820e").unwrap(),
    };


    let track_missing_metadata = Track {
        release_track_id: None,
        release_id: None,
        recording_id: uuid::Uuid::parse_str("fefd550f-b68e-4c11-b4d6-dfb4836a820e").unwrap(),
    };

    let another_track_missing_metadata = Track {
        release_track_id: None,
        release_id: None,
        recording_id: uuid::Uuid::parse_str("fefd550f-b68e-4c11-b4d6-dfb4836a820e").unwrap(),
    };

    let mut hashset = HashSet::new();

    hashset.insert(track_complete_metadata);
    assert_eq!(hashset.len(), 1);

    hashset.insert(another_track_complete_metadata);
    assert_eq!(hashset.len(), 1);

    hashset.insert(track_missing_metadata);
    assert_eq!(hashset.len(), 2);

    hashset.insert(another_track_missing_metadata);
    assert_eq!(hashset.len(), 2)
}
