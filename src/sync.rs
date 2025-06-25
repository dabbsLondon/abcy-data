use crate::{storage::Storage, strava::StravaApi, fit_parser::parse_fit_file};
use tracing::{debug, info};
use std::path::PathBuf;
use tokio::time::Duration;

pub async fn sync_latest<S: StravaApi + Sync>(client: &S, storage: &Storage, per_page: usize) -> anyhow::Result<()> {
    info!("checking for new activities");
    let activities = client.get_latest_activities(per_page).await?;
    debug!(count = activities.len(), "fetched activities from Strava");
    info!(count = activities.len(), "downloading fit files for activities");
    for act in activities {
        if storage.has_activity(act.id) {
            debug!(id = act.id, "activity parquet exists, skipping");
            continue;
        }

        storage.save_metadata(&act)?;
        let fit_path: PathBuf = storage.fit_file_path(act.id);

        if !storage.has_fit_file(act.id) {
            if client.download_fit(act.id, &fit_path).await.is_ok() {
                info!(id = act.id, "downloaded new fit file");
            }
        } else {
            debug!(id = act.id, "fit file already exists, skipping download");
        }

        if let Ok(points) = parse_fit_file(&fit_path) {
            storage.save_activity(act.id, &points)?;
        }
        info!(id = act.id, "processed activity");
    }
    Ok(())
}

pub async fn run_periodic_sync<S>(client: S, storage: Storage)
where
    S: StravaApi + Send + Sync + Clone + 'static,
{
    let client_clone = client.clone();
    info!("starting periodic sync task");
    // initial sync
    let _ = sync_latest(&client_clone, &storage, 10).await;
    let mut interval = tokio::time::interval(Duration::from_secs(300));
    loop {
        interval.tick().await;
        info!("running scheduled sync");
        let _ = sync_latest(&client, &storage, 10).await;
    }
}
