use abcy_data::{storage::Storage, utils::Storage as StorageCfg};
use serde_json::json;
use tempfile::tempdir;

fn make_storage() -> Storage {
    let dir = tempdir().unwrap();
    let cfg = StorageCfg { data_dir: dir.path().to_str().unwrap().into(), download_count: 1, user: "t".into() };
    Storage::new(&cfg)
}

#[tokio::test]
async fn summary_computation() {
    let storage = make_storage();
    let meta = json!({
        "id":1,
        "name":"ride",
        "start_date":"2024-01-01",
        "distance":1.0,
        "total_elevation_gain":10.0,
        "elapsed_time":30,
        "average_speed":0.033,
        "pr_count":2,
        "average_heartrate":95.0,
        "map": {"summary_polyline":"xyz"}
    });
    let streams = json!({
        "time": {"data": [0,10,20,30]},
        "watts": {"data": [100,150,200]},
        "heartrate": {"data": [90,100,95,95]}
    });
    storage.save(&meta, &streams).await.unwrap();
    let summary = storage.load_activity_summary(1).await.unwrap();
    assert_eq!(summary.id, 1);
    assert_eq!(summary.duration, 30);
    assert!(summary.weighted_average_power.unwrap() > 0.0);
    assert!(summary.average_speed.unwrap() > 0.0);
    assert_eq!(summary.pr_count, Some(2));
    assert!(summary.average_heartrate.unwrap() > 0.0);
    assert_eq!(summary.summary_polyline, Some("xyz".into()));
    assert_eq!(summary.total_elevation_gain, Some(10.0));
}
