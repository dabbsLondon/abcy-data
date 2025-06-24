use abcy_data::{config::Config, strava::StravaClient, storage::Storage, api, sync};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::from_env();
    let client = StravaClient::new(config.clone());
    let storage = Storage::new(&config.data_dir);

    // spawn background sync
    info!("spawning background sync task");
    tokio::spawn(sync::run_periodic_sync(client.clone(), storage.clone()));

    // start API
    info!("starting HTTP server");
    api::run_server(storage).await?;
    Ok(())
}
