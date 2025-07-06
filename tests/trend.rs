use abcy_data::{storage::Storage, utils::Storage as StorageCfg};
use serde_json::json;
use tempfile::tempdir;

fn make_storage() -> Storage {
    let dir = tempdir().unwrap();
    let cfg = StorageCfg { data_dir: dir.path().to_str().unwrap().into(), download_count: 1, user: "t".into() };
    Storage::new(&cfg)
}

async fn add_activity(storage: &Storage, id: u64, days_ago: i64, avg_speed: f64, max_speed: f64, tss: f64, intensity: f64, power: f64) {
    let dt = chrono::Utc::now().naive_utc().date() - chrono::Duration::days(days_ago);
    let date = dt.and_hms_opt(0, 0, 0).unwrap().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let meta = json!({
        "id": id,
        "name": "ride",
        "start_date": date,
        "distance": 10000.0,
        "elapsed_time": 1000,
        "average_speed": avg_speed,
        "max_speed": max_speed,
        "training_stress_score": tss,
        "intensity_factor": intensity,
        "weighted_average_watts": power
    });
    let streams = json!({"time": [0,1000]});
    storage.save(&meta, &streams).await.unwrap();
}

#[tokio::test]
async fn trend_computation() {
    let storage = make_storage();
    for i in 0..10 {
        add_activity(&storage, 20 + i as u64, 10 - i as i64, 11.0, 20.0, 105.0, 1.0, 230.0).await;
    }
    for i in 0..10 {
        add_activity(&storage, i as u64, 100 + i as i64, 10.0, 20.0, 100.0, 0.95, 200.0).await;
    }

    let trend = storage.recent_trends().await.unwrap();
    assert_eq!(trend.avg_speed, "high");
    assert_eq!(trend.max_speed, "same");
    assert_eq!(trend.tss, "high");
    assert_eq!(trend.intensity, "high");
    assert_eq!(trend.power, "high");
}
