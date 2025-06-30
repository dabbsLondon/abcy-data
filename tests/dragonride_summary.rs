use abcy_data::{storage::Storage, utils::Storage as StorageCfg};
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

fn make_storage() -> Storage {
    let dir = tempdir().unwrap();
    let cfg = StorageCfg { data_dir: dir.path().to_str().unwrap().into(), download_count: 1, user: "t".into() };
    Storage::new(&cfg)
}

#[tokio::test]
async fn dragonride_average_power() {
    let storage = make_storage();
    let data: Value = serde_json::from_str(&fs::read_to_string("dragonride.json").unwrap()).unwrap();
    let meta = &data["meta"];
    let streams = &data["streams"];
    storage.save(meta, streams).await.unwrap();
    let summary = storage.load_activity_summary(meta["id"].as_u64().unwrap()).await.unwrap();
    assert_eq!(summary.average_power.unwrap().round() as i64, 160);
}
