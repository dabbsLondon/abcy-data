use reqwest::Client;
use serde::Deserialize;
use std::path::Path;
use tokio::fs;
use crate::config::Config;
use async_trait::async_trait;

#[derive(Debug, Deserialize)]
pub struct ActivitySummary {
    pub id: u64,
    pub name: String,
    pub start_date: String,
    pub distance: f64,
}

#[derive(Clone)]
pub struct StravaClient {
    http: Client,
    config: Config,
}

impl StravaClient {
    pub fn new(config: Config) -> Self {
        Self { http: Client::new(), config }
    }
}

#[async_trait]
pub trait StravaApi {
    async fn get_latest_activities(&self, per_page: usize) -> anyhow::Result<Vec<ActivitySummary>>;
    async fn download_fit(&self, activity_id: u64, out_path: &Path) -> anyhow::Result<()>;
}

#[async_trait]
impl StravaApi for StravaClient {
    async fn get_latest_activities(&self, per_page: usize) -> anyhow::Result<Vec<ActivitySummary>> {
        let url = format!(
            "https://www.strava.com/api/v3/athlete/activities?per_page={}&access_token={}",
            per_page, self.config.strava_refresh_token
        );
        let resp = self.http.get(url).send().await?;
        let activities = resp.json::<Vec<ActivitySummary>>().await?;
        Ok(activities)
    }

    async fn download_fit(&self, activity_id: u64, out_path: &Path) -> anyhow::Result<()> {
        let url = format!(
            "https://www.strava.com/api/v3/activities/{}/export_original?access_token={}",
            activity_id, self.config.strava_refresh_token
        );
        let bytes = self.http.get(url).send().await?.bytes().await?;
        if let Some(dir) = out_path.parent() {
            fs::create_dir_all(dir).await?;
        }
        fs::write(out_path, &bytes).await?;
        Ok(())
    }
}
