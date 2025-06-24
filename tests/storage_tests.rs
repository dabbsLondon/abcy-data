use abcy_data::{storage::Storage, fit_parser::DataPoint};
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
