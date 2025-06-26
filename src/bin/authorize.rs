use anyhow::Context;
use reqwest::{Client, Url};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{error, info};

use abcy_data::utils::Config;

#[derive(Deserialize, serde::Serialize)]
struct Token {
    access_token: String,
    expires_at: i64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cfg = Config::load("config.toml")?;
    let client_id = &cfg.strava.client_id;
    let client_secret = &cfg.strava.client_secret;
    let token_path = &cfg.strava.token_path;

    let url = format!(
        "https://www.strava.com/oauth/authorize?client_id={}&response_type=code&redirect_uri=http://localhost:8080&approval_prompt=auto&scope=activity:read_all",
        client_id
    );
    webbrowser::open(&url).context("failed to open browser")?;
    info!("Opened browser to URL");

    let server = tiny_http::Server::http("0.0.0.0:8080").map_err(|e| {
        anyhow::anyhow!(
            "failed to bind to localhost:8080 (is it already running?): {}",
            e
        )
    })?;
    let request = server.recv().context("failed to receive request")?;
    let full_url = format!("http://localhost:8080{}", request.url());
    let parsed = Url::parse(&full_url)?;
    let params: HashMap<_, _> = parsed.query_pairs().into_owned().collect();
    let code = params
        .get("code")
        .cloned()
        .context("missing code in redirect")?;
    info!("Received redirect with code");

    let mut response = tiny_http::Response::from_string(
        "<html><body><h2>Authorization complete. You may close this window.</h2></body></html>",
    );
    response.add_header(
        tiny_http::Header::from_bytes("Content-Type", "text/html").unwrap(),
    );
    let _ = request.respond(response);

    let client = Client::new();
    let resp = client
        .post("https://www.strava.com/api/v3/oauth/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", &code),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .context("failed to send token request")?;
    let status = resp.status();
    if !status.is_success() {
        error!(%status, "token request failed");
        anyhow::bail!("token exchange failed");
    }
    let body = resp.text().await.context("failed to read token response")?;
    let token: Token = serde_json::from_str(&body).context("failed to parse token JSON")?;
    info!("Token exchange successful");
    info!("Access token: {}", token.access_token);
    info!("Expires at (unix): {}", token.expires_at);

    let sanitized = serde_json::to_vec(&token)?;
    tokio::fs::write(token_path, &sanitized).await?;
    info!("Token saved to {}", token_path);

    Ok(())
}

