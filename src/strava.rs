use reqwest::Client;
use serde::Deserialize;
use std::path::Path;
use tokio::fs;
use crate::config::Config;
use async_trait::async_trait;
use tracing::{debug, info};

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

    async fn access_token(&self) -> anyhow::Result<String> {
        if let Some(ref token) = self.config.strava_access_token {
            info!("using provided Strava access token");
            return Ok(token.clone());
        }

        #[derive(Deserialize)]
        struct TokenResp {
            access_token: String,
        }

        let resp = self
            .http
            .post("https://www.strava.com/oauth/token")
            .form(&[
                ("client_id", self.config.strava_client_id.as_str()),
                ("client_secret", self.config.strava_client_secret.as_str()),
                ("refresh_token", self.config.strava_refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await?;

        let status = resp.status();
        info!(status = %status, "token refresh response");
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            debug!(body, "unsuccessful token refresh");
            anyhow::bail!("strava token refresh returned {}", status);
        }

        let token = resp.json::<TokenResp>().await?.access_token;
        Ok(token)
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
            "https://www.strava.com/api/v3/athlete/activities?per_page={}",
            per_page
        );
        let token = self.access_token().await?;
        info!(per_page = per_page, request = %url, "requesting latest activities");
        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await?;
        let status = resp.status();
        info!(status = %status, "strava activities response");
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            debug!(body, "unsuccessful activities response");
            anyhow::bail!("strava returned {}", status);
        }
        let activities = resp.json::<Vec<ActivitySummary>>().await?;
        debug!(count = activities.len(), "received activities from Strava");
        Ok(activities)
    }

    async fn download_fit(&self, activity_id: u64, out_path: &Path) -> anyhow::Result<()> {
        let url = format!(
            "https://www.strava.com/api/v3/activities/{}/export_original",
            activity_id
        );
        let token = self.access_token().await?;
        info!(id = activity_id, request = %url, "downloading fit file");
        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await?;
        let status = resp.status();
        info!(id = activity_id, status = %status, "fit file response");
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            debug!(id = activity_id, body, "unsuccessful fit download");
            anyhow::bail!("strava returned {}", status);
        }
        let bytes = resp.bytes().await?;
        if let Some(dir) = out_path.parent() {
            fs::create_dir_all(dir).await?;
        }
        fs::write(out_path, &bytes).await?;
        debug!(id = activity_id, path = %out_path.display(), "fit file saved");
        Ok(())
    }
}
