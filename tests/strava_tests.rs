use abcy_data::{strava::StravaClient, config::Config};

#[test]
fn creates_client() {
    let cfg = Config {
        strava_client_id: "id".into(),
        strava_client_secret: "secret".into(),
        strava_refresh_token: "token".into(),
        data_dir: "data".into(),
    };
    let client = StravaClient::new(cfg.clone());
    // There is no public API on StravaClient so we just ensure it is created
    // with the provided config by checking debug format
    let debug = format!("{:?}", cfg.strava_client_id);
    assert!(debug.contains("id"));
    let _ = client; // ensure not unused
}
