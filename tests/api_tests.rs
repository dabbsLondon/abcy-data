use actix_web::{test, App};
use abcy_data::storage::Storage;
use abcy_data::api::{list_activities, list_raw_files, download_fit, fit_details};
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

#[actix_rt::test]
async fn list_raw_files_returns_empty() {
    let dir = tempdir().unwrap();
    let storage = Storage::new(dir.path());

    let app = test::init_service(App::new().app_data(actix_web::web::Data::new(storage)).service(list_raw_files)).await;
    let req = test::TestRequest::get().uri("/raw").to_request();
    let resp: Vec<serde_json::Value> = test::call_and_read_body_json(&app, req).await;
    assert!(resp.is_empty());
}

#[actix_rt::test]
async fn download_fit_missing_returns_404() {
    let dir = tempdir().unwrap();
    let storage = Storage::new(dir.path());
    let app = test::init_service(App::new().app_data(actix_web::web::Data::new(storage)).service(download_fit)).await;
    let req = test::TestRequest::get().uri("/fit/1").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_rt::test]
async fn fit_details_missing_returns_404() {
    let dir = tempdir().unwrap();
    let storage = Storage::new(dir.path());
    let app = test::init_service(App::new().app_data(actix_web::web::Data::new(storage)).service(fit_details)).await;
    let req = test::TestRequest::get().uri("/fit/1/details").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}
