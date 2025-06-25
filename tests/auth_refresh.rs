use abcy_data::{auth::Auth, utils::{Config, Strava, Storage}};
use mockito::Server;

#[tokio::test(flavor = "current_thread")]
async fn refresh_on_401() {
    let mut server = Server::new_async().await;
    let _m1 = server.mock("GET", "/athlete/activities").with_status(401).create();
    let _refresh = server
        .mock("POST", "/oauth/token")
        .with_status(200)
        .with_body("{\"access_token\": \"newtok\"}")
        .create();
    let _m2 = server
        .mock("GET", "/athlete/activities")
        .with_status(200)
        .with_body("[]")
        .create();

    let cfg = Config {
        strava: Strava {
            client_id: "id".into(),
            client_secret: "secret".into(),
            refresh_token: "refresh".into(),
            token_path: "token.json".into(),
        },
        storage: Storage { data_dir: "tmp".into(), download_count: 1, user: "t".into() },
        base_url: server.url(),
    };
    tokio::fs::write("token.json", "{\"access_token\":\"expired\"}").await.unwrap();
    let auth = Auth::new(cfg);
    let _ = auth.get_json::<serde_json::Value>(&format!("{}/athlete/activities", server.url())).await.unwrap();
    assert!(_refresh.matched());
}
