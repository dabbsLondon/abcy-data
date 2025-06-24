use abcy_data::config::Config;
use std::env;

#[test]
fn from_env_reads_variables() {
    env::set_var("STRAVA_CLIENT_ID", "id");
    env::set_var("STRAVA_CLIENT_SECRET", "secret");
    env::set_var("STRAVA_REFRESH_TOKEN", "token");
    env::set_var("DATA_DIR", "tmpdata");

    let cfg = Config::from_env();
    assert_eq!(cfg.strava_client_id, "id");
    assert_eq!(cfg.strava_client_secret, "secret");
    assert_eq!(cfg.strava_refresh_token, "token");
    assert_eq!(cfg.data_dir, "tmpdata");
}
