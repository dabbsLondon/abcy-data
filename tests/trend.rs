use abcy_data::{storage::Storage, utils::Storage as StorageCfg};
use serde_json::json;
use tempfile::tempdir;

fn make_storage() -> Storage {
    let dir = tempdir().unwrap();
    let cfg = StorageCfg { data_dir: dir.path().to_str().unwrap().into(), download_count: 1, user: "t".into() };
    Storage::new(&cfg)
}

async fn add_activity(storage: &Storage, id: u64, day: u64, avg_speed: f64, max_speed: f64, tss: f64, intensity: f64, power: f64) {
    let date = format!("2024-01-{:02}T00:00:00Z", day);
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
        add_activity(&storage, i as u64, i as u64 + 1, 10.0, 20.0, 100.0, 0.95, 200.0).await;
    }
    for i in 10..20 {
        add_activity(&storage, i as u64, i as u64 + 1, 11.0, 20.0, 105.0, 1.0, 230.0).await;
    }

    let trend = storage.recent_trends().await.unwrap();
    assert_eq!(trend.avg_speed, "very_high");
    assert_eq!(trend.max_speed, "normal");
    assert_eq!(trend.tss, "high");
    assert_eq!(trend.intensity, "high");
    assert_eq!(trend.power, "very_high");
}
