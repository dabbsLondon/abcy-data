use abcy_data::{storage::Storage, utils::Storage as StorageCfg, stats::Period};
use serde_json::json;
use tempfile::tempdir;

fn make_storage() -> Storage {
    let dir = tempdir().unwrap();
    let cfg = StorageCfg { data_dir: dir.path().to_str().unwrap().into(), download_count: 1, user: "t".into() };
    Storage::new(&cfg)
}

#[tokio::test]
async fn stats_aggregation() {
    let storage = make_storage();
    let meta1 = json!({"id":1,"name":"r1","start_date":"2024-01-01T00:00:00Z","distance":10.0,"type":"Ride"});
    let streams1 = json!({"time": [0,10], "watts": [100,200]});
    storage.save(&meta1, &streams1).await.unwrap();

    let meta2 = json!({"id":2,"name":"r2","start_date":"2024-01-02T00:00:00Z","distance":5.0,"type":"Run"});
    let streams2 = json!({"time": [0,10], "watts": [150,250]});
    storage.save(&meta2, &streams2).await.unwrap();

    let stats = storage.activity_stats(Period::Day, None, None).await.unwrap();
    assert_eq!(stats.len(), 2);
    assert!(stats.iter().any(|s| s.distance > 9.0));

    let filtered = storage.activity_stats(Period::Day, Some(&[1]), None).await.unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].rides, 1);

    let type_filtered = storage
        .activity_stats(Period::Day, None, Some(&["Ride".to_string()]))
        .await
        .unwrap();
    assert_eq!(type_filtered.len(), 1);
    assert_eq!(type_filtered[0].rides, 1);
}

