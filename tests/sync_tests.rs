use abcy_data::{storage::Storage, strava::{ActivitySummary, StravaApi}, sync};
use tempfile::tempdir;
use async_trait::async_trait;
use std::path::Path;
use std::sync::{Arc, Mutex};

struct DummyClient {
    downloads: Arc<Mutex<usize>>,
}

impl DummyClient {
    fn new() -> Self {
        Self { downloads: Arc::new(Mutex::new(0)) }
    }
}

#[async_trait]
impl StravaApi for DummyClient {
    async fn get_latest_activities(&self, _per_page: usize) -> anyhow::Result<Vec<ActivitySummary>> {
        Ok(vec![ActivitySummary { id: 99, name: "x".into(), start_date: "".into(), distance: 0.0 }])
    }

    async fn download_fit(&self, _activity_id: u64, out_path: &Path) -> anyhow::Result<()> {
        *self.downloads.lock().unwrap() += 1;
        tokio::fs::write(out_path, b"dummy").await?;
        Ok(())
    }
}

#[tokio::test]
async fn sync_downloads_missing_activity() {
    let dir = tempdir().unwrap();
    let storage = Storage::new(dir.path());
    let client = DummyClient::new();
    sync::sync_latest(&client, &storage, 10).await.unwrap();
    assert!(dir.path().join("raw/99.fit").exists());
    assert!(dir.path().join("metadata.parquet").exists());
    assert_eq!(*client.downloads.lock().unwrap(), 1);
}

#[tokio::test]
async fn sync_skips_existing_activity() {
    let dir = tempdir().unwrap();
    let storage = Storage::new(dir.path());
    std::fs::create_dir_all(dir.path().join("raw")).unwrap();
    std::fs::write(dir.path().join("99.parquet"), b"x").unwrap();
    let client = DummyClient::new();
    sync::sync_latest(&client, &storage, 10).await.unwrap();
    assert_eq!(*client.downloads.lock().unwrap(), 0);
}
