use crate::auth::Auth;
use crate::schema::ActivityHeader;
use crate::storage::Storage;
use tracing::{error, info};

pub async fn download_latest(auth: &Auth, storage: &Storage, count: usize) -> anyhow::Result<()> {
    let url = format!("{}/athlete/activities?per_page={}", auth.cfg.base_url, count);
    info!("Requesting activity list: {}", url);
    let acts: Vec<ActivityHeader> = auth.get_json(&url).await?;
    for summary in acts {
        info!(id = summary.id, name = %summary.name, "download activity");
        let meta_url = format!("{}/activities/{}", auth.cfg.base_url, summary.id);
        info!("Requesting activity metadata: {}", meta_url);
        let meta: serde_json::Value = auth.get_json(&meta_url).await?;
        let streams_url = format!("{}/activities/{}/streams?keys=latlng,time,altitude,heartrate,watts&key_by_type=true", auth.cfg.base_url, summary.id);
        info!("Requesting activity streams: {}", streams_url);
        let streams: serde_json::Value = auth.get_json(&streams_url).await?;
        if let Err(e) = storage.save(&meta, &streams).await {
            error!(?e, "failed to save activity");
        }
    }
    Ok(())
}
