use abcy_data::{storage::Storage, utils::Storage as StorageCfg};
use serde_json::json;
use tempfile::tempdir;

fn make_storage() -> Storage {
    let dir = tempdir().unwrap();
    let cfg = StorageCfg { data_dir: dir.path().to_str().unwrap().into(), download_count: 1, user: "t".into() };
    Storage::new(&cfg)
}

#[tokio::test]
async fn activity_exists_check() {
    let storage = make_storage();
    assert!(!storage.activity_exists("2024", 1).await);
    let meta = json!({"id":1,"name":"ride","start_date":"2024-01-01","distance":1.0});
    let streams = json!({"time":[1,2,3]});
    storage.save(&meta, &streams).await.unwrap();
    assert!(storage.activity_exists("2024", 1).await);
}
