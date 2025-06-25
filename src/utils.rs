use serde::Deserialize;
use std::fs;
use anyhow::Context;

#[derive(Debug, Clone, Deserialize)]
pub struct Strava {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: Option<String>,
    pub token_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Storage {
    pub data_dir: String,
    pub download_count: usize,
    pub user: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub strava: Strava,
    pub storage: Storage,
    #[serde(default = "default_base_url")]
    pub base_url: String,
}

fn default_base_url() -> String {
    "https://www.strava.com/api/v3".into()
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        tracing::info!("loading configuration from {}", path);
        let text = fs::read_to_string(path)
            .with_context(|| format!("config file '{}' not found", path))?;
        let cfg: Self = toml::from_str(&text)
            .context("failed to parse TOML in config file")?;
        Ok(cfg)
    }
}
