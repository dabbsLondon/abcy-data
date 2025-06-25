use abcy_data::{utils::Config, auth::Auth, storage::Storage, fetch, web};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = Config::load("config.toml")?;
    let auth = Auth::new(cfg.clone());
    let storage = Storage::new(&cfg.storage);
    // initial download
    info!("downloading latest activities");
    let _ = fetch::download_latest(&auth, &storage, cfg.storage.download_count).await;
    web::run(cfg, auth, storage).await?;
    Ok(())
}
