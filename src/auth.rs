use crate::utils::Config;
use reqwest::{Client, Method};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Clone)]
pub struct Auth {
    client: Client,
    pub cfg: Config,
    token: Arc<Mutex<Option<String>>>,
}

impl Auth {
    pub fn new(cfg: Config) -> Self {
        Self { client: Client::new(), cfg, token: Arc::new(Mutex::new(None)) }
    }

    async fn load_token(&self) -> Option<String> {
        let path = &self.cfg.strava.token_path;
        match tokio::fs::read_to_string(path).await {
            Ok(t) => serde_json::from_str::<Token>(&t).ok().map(|t| t.access_token),
            Err(_) => None,
        }
    }

    async fn save_token(&self, token: &str) -> anyhow::Result<()> {
        let path = &self.cfg.strava.token_path;
        let data = serde_json::to_string(&Token { access_token: token.into() })?;
        tokio::fs::write(path, data).await?;
        Ok(())
    }

    async fn ensure_token(&self) -> anyhow::Result<String> {
        if let Some(t) = self.token.lock().await.clone() {
            return Ok(t);
        }
        if let Some(t) = self.load_token().await { 
            *self.token.lock().await = Some(t.clone());
            return Ok(t);
        }
        let t = self.refresh_token().await?;
        Ok(t)
    }

    pub async fn refresh_token(&self) -> anyhow::Result<String> {
        #[derive(Deserialize)]
        struct Resp { access_token: String }
        let resp = self.client.post(format!("{}/oauth/token", self.cfg.base_url.trim_end_matches("/api/v3")))
            .form(&[
                ("client_id", self.cfg.strava.client_id.as_str()),
                ("client_secret", self.cfg.strava.client_secret.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", self.cfg.strava.refresh_token.as_str()),
            ])
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() { warn!(%status, "token refresh failed"); anyhow::bail!("refresh failed") }
        let token = resp.json::<Resp>().await?.access_token;
        info!("refreshed strava token");
        *self.token.lock().await = Some(token.clone());
        self.save_token(&token).await?;
        Ok(token)
    }

    pub async fn request(&self, method: Method, url: &str) -> anyhow::Result<reqwest::Response> {
        let token = self.ensure_token().await?;
        let req = self.client.request(method.clone(), url).bearer_auth(&token);
        let mut resp = req.try_clone().unwrap().send().await?;
        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            warn!("401 from Strava, refreshing token");
            let token = self.refresh_token().await?;
            resp = self.client.request(method, url).bearer_auth(&token).send().await?;
        }
        Ok(resp)
    }

    pub async fn get_json<T: for<'de> serde::Deserialize<'de>>(&self, url: &str) -> anyhow::Result<T> {
        let resp = self.request(Method::GET, url).await?;
        Ok(resp.json::<T>().await?)
    }
}

#[derive(Deserialize, serde::Serialize)]
struct Token { access_token: String }
