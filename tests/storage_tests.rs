use abcy_data::{storage::Storage, fit_parser::DataPoint, strava::ActivitySummary};
use tempfile::tempdir;

#[test]
fn save_activity_creates_parquet() {
    let dir = tempdir().unwrap();
    let storage = Storage::new(dir.path());

    let points = vec![DataPoint { timestamp: Some(1), power: Some(2), heart_rate: None, cadence: None, distance: None }];

    storage.save_activity(42, &points).unwrap();
    let file = dir.path().join("42.parquet");
    assert!(file.exists());
    assert!(storage.has_activity(42));
}

#[test]
fn save_metadata_creates_file() {
    let dir = tempdir().unwrap();
    let storage = Storage::new(dir.path());
    let summary = ActivitySummary { id: 1, name: "Ride".into(), start_date: "2023-01-01".into(), distance: 1.0 };
    storage.save_metadata(&summary).unwrap();
    let file = dir.path().join("metadata.parquet");
    assert!(file.exists());
    assert!(storage.has_metadata(1));
}
