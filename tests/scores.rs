use abcy_data::{storage::Storage, utils::Storage as StorageCfg};
use serde_json::json;
use tempfile::tempdir;
use chrono::{Utc, Duration};

fn make_storage() -> Storage {
    let dir = tempdir().unwrap();
    let cfg = StorageCfg { data_dir: dir.path().to_str().unwrap().into(), download_count: 1, user: "t".into() };
    Storage::new(&cfg)
}

async fn add_activity(storage: &Storage, id: u64, days_ago: i64, distance: f64, duration: i64, tss: f64) {
    let date = (Utc::now() - Duration::days(days_ago)).format("%Y-%m-%dT00:00:00Z").to_string();
    let meta = json!({
        "id": id,
        "name": "ride",
        "start_date": date,
        "distance": distance,
        "elapsed_time": duration,
        "training_stress_score": tss
    });
    let streams = json!({"time": [0, duration]});
    storage.save(&meta, &streams).await.unwrap();
}

#[tokio::test]
async fn enduro_score_computation() {
    let storage = make_storage();
    add_activity(&storage, 1, 1, 100000.0, 14400, 200.0).await;
    add_activity(&storage, 2, 10, 100000.0, 10800, 180.0).await;
    add_activity(&storage, 3, 3, 50000.0, 7200, 100.0).await;

    let score = storage.update_enduro().await.unwrap();
    let avg_long = (100000.0 * 14400.0 + 100000.0 * 10800.0) / 2.0;
    let expected = avg_long / 10000.0 + 6.0 + 4.8;
    assert!((score - expected).abs() < 1e-6);
    let hist = storage.enduro_history(None).await.unwrap();
    assert_eq!(hist.len(), 1);
    assert!((hist[0].score - score).abs() < 1e-6);
}

#[tokio::test]
async fn enduro_score_decay() {
    let storage = make_storage();
    add_activity(&storage, 1, 20, 100000.0, 14400, 200.0).await;
    add_activity(&storage, 2, 1, 50000.0, 7200, 100.0).await;

    let score = storage.update_enduro().await.unwrap();
    let base = 144000.0 + 2.0 + 3.0;
    let expected = base * 0.9_f64.powf(6.0);
    assert!((score - expected).abs() < 1e-6);
}

#[tokio::test]
async fn fitness_score_computation() {
    let storage = make_storage();
    add_activity(&storage, 1, 1, 100000.0, 14400, 200.0).await;
    add_activity(&storage, 2, 10, 100000.0, 10800, 180.0).await;
    add_activity(&storage, 3, 3, 50000.0, 7200, 100.0).await;

    let score = storage.update_fitness().await.unwrap();
    let expected = 6.0 * 4.0 + (480.0 / 4.0) / 10.0 + 2.0;
    assert!((score - expected).abs() < 1e-6);
}

#[tokio::test]
async fn fitness_score_decay() {
    let storage = make_storage();
    add_activity(&storage, 1, 20, 100000.0, 14400, 200.0).await;
    add_activity(&storage, 2, 15, 30000.0, 3600, 50.0).await;
    add_activity(&storage, 3, 7, 30000.0, 3600, 50.0).await;

    let score = storage.update_fitness().await.unwrap();
    let base = (300.0 / 4.0) / 10.0 + 1.0;
    let expected = base * 0.985_f64.powf(4.0);
    assert!((score - expected).abs() < 1e-6);
}

