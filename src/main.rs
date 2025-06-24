use abcy_data::{config::Config, strava::StravaClient, fit_parser::parse_fit_file, storage::Storage, api};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::from_env();
    let client = StravaClient::new(config.clone());
    let storage = Storage::new(&config.data_dir);

    // Example workflow: list activities and download first one
    let activities = client.get_latest_activities().await?;
    if let Some(act) = activities.first() {
        let fit_path = format!("{}/{}.fit", config.data_dir, act.id);
        if client.download_fit(act.id, std::path::Path::new(&fit_path)).await.is_ok() {
            if let Ok(points) = parse_fit_file(std::path::Path::new(&fit_path)) {
                storage.save_activity(act.id, &points)?;
            }
        }
    }

    // start API
    api::run_server(storage).await?;
    Ok(())
}
