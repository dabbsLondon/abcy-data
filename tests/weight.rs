use abcy_data::{storage::Storage, utils::Storage as StorageCfg};
use tempfile::tempdir;

fn make_storage() -> Storage {
    let dir = tempdir().unwrap();
    let cfg = StorageCfg { data_dir: dir.path().to_str().unwrap().into(), download_count: 1, user: "t".into() };
    Storage::new(&cfg)
}

#[tokio::test]
async fn weight_and_wkg_update() {
    let storage = make_storage();
    // default weight entry
    let hist1 = storage.get_weight_history().await.unwrap();
    assert_eq!(hist1.len(), 1);
    assert_eq!(hist1[0].weight, 75.0);

    storage.set_weight(83.5).await.unwrap();
    let current = storage.current_weight().await.unwrap();
    assert_eq!(current, 83.5);

    let hist2 = storage.get_weight_history().await.unwrap();
    assert_eq!(hist2.len(), 2);
    assert_eq!(hist2.last().unwrap().weight, 83.5);

    let wkg = storage.current_wkg().await.unwrap();
    assert!((wkg - 240.0 / 83.5).abs() < 1e-6);

    let wkg_hist = storage.wkg_history(Some(1)).await.unwrap();
    assert_eq!(wkg_hist.len(), 1);
}
