use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub strava_client_id: String,
    pub strava_client_secret: String,
    pub strava_refresh_token: String,
    pub strava_access_token: Option<String>,
    pub data_dir: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            strava_client_id: env::var("STRAVA_CLIENT_ID").expect("STRAVA_CLIENT_ID not set"),
            strava_client_secret: env::var("STRAVA_CLIENT_SECRET").expect("STRAVA_CLIENT_SECRET not set"),
            strava_refresh_token: env::var("STRAVA_REFRESH_TOKEN").expect("STRAVA_REFRESH_TOKEN not set"),
            strava_access_token: env::var("STRAVA_ACCESS_TOKEN").ok(),
            data_dir: env::var("DATA_DIR").unwrap_or_else(|_| "data".into()),
        }
    }
}
