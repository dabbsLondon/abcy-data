use abcy_data::{storage::Storage, utils::Storage as StorageCfg};
use tempfile::tempdir;

fn make_storage() -> Storage {
    let dir = tempdir().unwrap();
    let cfg = StorageCfg { data_dir: dir.path().to_str().unwrap().into(), download_count: 1, user: "t".into() };
    Storage::new(&cfg)
}

#[tokio::test]
async fn ftp_history_update() {
    let storage = make_storage();
    // first call creates default entry
    let hist1 = storage.get_ftp_history().await.unwrap();
    assert_eq!(hist1.len(), 1);
    assert_eq!(hist1[0].ftp, 240.0);

    // set new ftp
    storage.set_ftp(250.0).await.unwrap();
    let current = storage.current_ftp().await.unwrap();
    assert_eq!(current, 250.0);

    let hist2 = storage.get_ftp_history().await.unwrap();
    assert_eq!(hist2.len(), 2);
    assert_eq!(hist2.last().unwrap().ftp, 250.0);

    let recent = storage.ftp_history(Some(1)).await.unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].ftp, 250.0);
}
