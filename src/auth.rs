use crate::utils::Config;
use anyhow::Context;
use reqwest::{Client, Method, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct Auth {
    client: Client,
    pub cfg: Config,
    token: Arc<Mutex<Option<Token>>>,
}

impl Auth {
    pub fn new(cfg: Config) -> Self {
        Self { client: Client::new(), cfg, token: Arc::new(Mutex::new(None)) }
    }

    async fn load_token(&self) -> Option<Token> {
        let path = &self.cfg.strava.token_path;
        match tokio::fs::read_to_string(path).await {
            Ok(t) => serde_json::from_str::<Token>(&t).ok(),
            Err(_) => None,
        }
    }

    async fn save_token(&self, token: &Token) -> anyhow::Result<()> {
        let path = &self.cfg.strava.token_path;
        let data = serde_json::to_string(token)?;
        tokio::fs::write(path, data).await?;
        Ok(())
    }

    async fn ensure_token(&self) -> anyhow::Result<String> {
        if let Some(tok) = self.token.lock().await.clone() {
            if tok.expires_at > chrono::Utc::now().timestamp() {
                return Ok(tok.access_token.clone());
            }
            return Ok(self.refresh_token_with(&tok.refresh_token).await?.access_token);
        }
        if let Some(tok) = self.load_token().await {
            if tok.expires_at > chrono::Utc::now().timestamp() {
                *self.token.lock().await = Some(tok.clone());
                return Ok(tok.access_token);
            }
            return Ok(self.refresh_token_with(&tok.refresh_token).await?.access_token);
        }
        if let Some(ref rt) = self.cfg.strava.refresh_token {
            return Ok(self.refresh_token_with(rt).await?.access_token);
        }
        let tok = self.authorize().await?;
        Ok(tok.access_token)
    }

    async fn refresh_token_with(&self, refresh_token: &str) -> anyhow::Result<Token> {
        #[derive(Deserialize)]
        struct Resp { access_token: String, refresh_token: String, expires_at: i64 }
        let resp = self
            .client
            .post(format!("{}/oauth/token", self.cfg.base_url.trim_end_matches("/api/v3")))
            .form(&[
                ("client_id", self.cfg.strava.client_id.as_str()),
                ("client_secret", self.cfg.strava.client_secret.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            warn!(%status, "token refresh failed");
            anyhow::bail!("refresh failed")
        }
        let resp = resp.json::<Resp>().await?;
        info!(expires_at = resp.expires_at, "refreshed strava token");
        let token = Token {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            expires_at: resp.expires_at,
        };
        *self.token.lock().await = Some(token.clone());
        self.save_token(&token).await?;
        Ok(token)
    }

    async fn authorize(&self) -> anyhow::Result<Token> {
        let url = format!(
            "https://www.strava.com/oauth/authorize?client_id={}&response_type=code&redirect_uri=http://localhost:8080&approval_prompt=auto&scope=activity:read_all",
            self.cfg.strava.client_id
        );
        webbrowser::open(&url).context("failed to open browser")?;
        info!("Opened browser to URL: {}", url);

        let server = tiny_http::Server::http("0.0.0.0:8080").map_err(|e| {
            anyhow::anyhow!("failed to bind to localhost:8080 (is it already running?): {}", e)
        })?;
        let request = server.recv().context("failed to receive request")?;
        let full_url = format!("http://localhost:8080{}", request.url());
        let parsed = Url::parse(&full_url)?;
        let params: HashMap<_, _> = parsed.query_pairs().into_owned().collect();
        let code = params
            .get("code")
            .cloned()
            .context("missing code in redirect")?;
        info!("Received redirect with code: {}", code);

        let mut response = tiny_http::Response::from_string(
            "<html><body><h2>Authorization complete. You may close this window.</h2></body></html>",
        );
        response.add_header(tiny_http::Header::from_bytes("Content-Type", "text/html").unwrap());
        let _ = request.respond(response);

        #[derive(Deserialize)]
        struct Resp { access_token: String, refresh_token: String, expires_at: i64 }
        let resp = self
            .client
            .post("https://www.strava.com/api/v3/oauth/token")
            .form(&[
                ("client_id", self.cfg.strava.client_id.as_str()),
                ("client_secret", self.cfg.strava.client_secret.as_str()),
                ("code", &code),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            error!(%status, "token request failed");
            anyhow::bail!("token exchange failed")
        }
        let tok = resp.json::<Resp>().await?;
        info!("Token exchange successful");
        info!("Access token: {}", tok.access_token);
        info!("Refresh token: {}", tok.refresh_token);
        info!("Expires at (unix): {}", tok.expires_at);
        let token = Token {
            access_token: tok.access_token,
            refresh_token: tok.refresh_token,
            expires_at: tok.expires_at,
        };
        *self.token.lock().await = Some(token.clone());
        self.save_token(&token).await?;
        Ok(token)
    }

    pub async fn request(&self, method: Method, url: &str) -> anyhow::Result<reqwest::Response> {
        let token = self.ensure_token().await?;
        info!("HTTP {} {} with bearer {}", method.as_str(), url, token);
        let req = self.client.request(method.clone(), url).bearer_auth(&token);
        let mut resp = req.try_clone().unwrap().send().await?;
        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            warn!("401 from Strava, refreshing token");
            let token = if let Some(tok) = self.token.lock().await.clone() {
                self.refresh_token_with(&tok.refresh_token).await?
            } else if let Some(ref rt) = self.cfg.strava.refresh_token {
                self.refresh_token_with(rt).await?
            } else {
                self.authorize().await?
            };
            info!("Retrying {} {} with bearer {}", method.as_str(), url, token.access_token);
            resp = self
                .client
                .request(method, url)
                .bearer_auth(&token.access_token)
                .send()
                .await?;
        }
        Ok(resp)
    }

    pub async fn get_json<T: for<'de> serde::Deserialize<'de>>(&self, url: &str) -> anyhow::Result<T> {
        let resp = self.request(Method::GET, url).await?;
        Ok(resp.json::<T>().await?)
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct Token {
    access_token: String,
    refresh_token: String,
    expires_at: i64,
}
