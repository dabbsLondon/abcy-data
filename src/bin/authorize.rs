use anyhow::Context;
use reqwest::{Client, Url};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{error, info};

#[derive(Deserialize, serde::Serialize)]
struct Token {
    access_token: String,
    refresh_token: String,
    expires_at: i64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client_id = "165730";
    let client_secret = "81a257622cb85c052d11065d08d399bd9dda83ea";

    let url = format!(
        "https://www.strava.com/oauth/authorize?client_id={}&response_type=code&redirect_uri=http://localhost:8080&approval_prompt=auto&scope=activity:read_all",
        client_id
    );
    webbrowser::open(&url).context("failed to open browser")?;
    info!("Opened browser to URL");

    let server = tiny_http::Server::http("0.0.0.0:8080").unwrap();
    let request = server.recv().context("failed to receive request")?;
    let full_url = format!("http://localhost:8080{}", request.url());
    let parsed = Url::parse(&full_url)?;
    let params: HashMap<_, _> = parsed.query_pairs().into_owned().collect();
    let code = params
        .get("code")
        .cloned()
        .context("missing code in redirect")?;
    info!("Received redirect with code");

    let response = tiny_http::Response::from_string("Authorization complete. You may close this tab.");
    let _ = request.respond(response);

    let client = Client::new();
    let resp = client
        .post("https://www.strava.com/api/v3/oauth/token")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
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
    let token: Token = resp.json().await.context("failed to parse token JSON")?;
    info!("Token exchange successful");

    let json = serde_json::to_string_pretty(&token)?;
    tokio::fs::write("strava_tokens.json", json).await?;
    info!("Token saved to strava_tokens.json");

    Ok(())
}

