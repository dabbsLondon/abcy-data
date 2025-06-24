use reqwest::Client;
use serde::Deserialize;
use std::path::Path;
use tokio::fs;
use crate::config::Config;

#[derive(Debug, Deserialize)]
pub struct ActivitySummary {
    pub id: u64,
    pub name: String,
    pub start_date: String,
    pub distance: f64,
}

pub struct StravaClient {
    http: Client,
    config: Config,
}

impl StravaClient {
    pub fn new(config: Config) -> Self {
        Self { http: Client::new(), config }
    }

    pub async fn get_latest_activities(&self) -> anyhow::Result<Vec<ActivitySummary>> {
        let url = format!("https://www.strava.com/api/v3/athlete/activities?access_token={}", self.config.strava_refresh_token);
        let resp = self.http.get(url).send().await?;
        let activities = resp.json::<Vec<ActivitySummary>>().await?;
        Ok(activities)
    }

    pub async fn download_fit(&self, activity_id: u64, out_path: &Path) -> anyhow::Result<()> {
        let url = format!("https://www.strava.com/api/v3/activities/{}/export_original?access_token={}", activity_id, self.config.strava_refresh_token);
        let bytes = self.http.get(url).send().await?.bytes().await?;
        if let Some(dir) = out_path.parent() { fs::create_dir_all(dir).await?; }
        fs::write(out_path, &bytes).await?;
        Ok(())
    }
}
