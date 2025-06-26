use abcy_data::{storage::Storage, utils::Storage as StorageCfg, schema::ParsedStreams};
use serde_json::json;
use tempfile::tempdir;

fn make_storage() -> Storage {
    let dir = tempdir().unwrap();
    let cfg = StorageCfg { data_dir: dir.path().to_str().unwrap().into(), download_count: 1, user: "t".into() };
    Storage::new(&cfg)
}

#[tokio::test]
async fn round_trip() {
    let storage = make_storage();
    let meta = json!({"id":1,"name":"ride","start_date":"2024-01-01","distance":1.0});
    let streams = json!({"time": {"data": [1,2,3]}, "watts": {"data": [10,20]}});
    storage.save(&meta, &streams).await.unwrap();
    let act = storage.load_activity(1).await.unwrap();
    assert_eq!(act.meta, meta);
    assert_eq!(
        act.streams,
        ParsedStreams { time: vec![1,2,3], power: vec![10,20] }
    );
}
