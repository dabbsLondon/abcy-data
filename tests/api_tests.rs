use actix_web::{test, App};
use abcy_data::storage::Storage;
use abcy_data::api::list_activities; // we need to import for service
use tempfile::tempdir;

#[actix_rt::test]
async fn list_activities_returns_empty() {
    let dir = tempdir().unwrap();
    let storage = Storage::new(dir.path());

    let app = test::init_service(App::new().app_data(actix_web::web::Data::new(storage)).service(list_activities)).await;
    let req = test::TestRequest::get().uri("/activities").to_request();
    let resp: Vec<serde_json::Value> = test::call_and_read_body_json(&app, req).await;
    assert!(resp.is_empty());
}
