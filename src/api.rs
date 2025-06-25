use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use crate::storage::Storage;
use crate::{strava::StravaClient, sync, fit_parser::parse_fit_file};
use std::fs;
use std::ffi::OsStr;
use std::collections::HashMap;

#[derive(Serialize)]
struct ApiActivity {
    id: u64,
}

#[derive(Serialize)]
struct ApiFile {
    name: String,
}

#[get("/activities")]
async fn list_activities(storage: web::Data<Storage>) -> impl Responder {
    let mut ids = Vec::new();
    if let Ok(entries) = fs::read_dir(&storage.data_dir) {
        for entry in entries.flatten() {
            if let Some(stem) = entry.path().file_stem() {
                if let Ok(id) = stem.to_string_lossy().parse::<u64>() {
                    ids.push(ApiActivity { id });
                }
            }
        }
    }
    web::Json(ids)
}

#[get("/raw")]
async fn list_raw_files(storage: web::Data<Storage>) -> impl Responder {
    let mut files = Vec::new();
    let raw_dir = storage.data_dir.join("raw");
    if let Ok(entries) = fs::read_dir(&raw_dir) {
        for entry in entries.flatten() {
            if entry.path().extension() == Some(OsStr::new("fit")) {
                if let Some(name) = entry.path().file_name() {
                    files.push(ApiFile { name: name.to_string_lossy().into() });
                }
            }
        }
    }
    web::Json(files)
}

#[get("/webhook")]
async fn webhook_verify(query: web::Query<HashMap<String, String>>) -> impl Responder {
    if let Some(ch) = query.get("hub.challenge") {
        HttpResponse::Ok().json(serde_json::json!({"hub.challenge": ch}))
    } else {
        HttpResponse::BadRequest().finish()
    }
}

#[derive(Deserialize)]
struct WebhookEvent {
    #[allow(dead_code)]
    object_type: String,
    #[allow(dead_code)]
    aspect_type: String,
}

#[post("/webhook")]
async fn webhook_event(
    client: web::Data<StravaClient>,
    storage: web::Data<Storage>,
    event: web::Json<WebhookEvent>,
) -> impl Responder {
    if event.object_type == "activity" && event.aspect_type == "create" {
        let client = client.clone();
        let storage = storage.clone();
        tokio::spawn(async move {
            let _ = sync::sync_latest::<StravaClient>(&*client, &storage, 1).await;
        });
    }
    HttpResponse::Ok()
}

#[get("/fit/{id}")]
async fn download_fit(path: web::Path<u64>, storage: web::Data<Storage>) -> impl Responder {
    let fit_path = storage.fit_file_path(*path);
    match fs::read(&fit_path) {
        Ok(data) => HttpResponse::Ok().body(data),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[get("/fit/{id}/details")]
async fn fit_details(path: web::Path<u64>, storage: web::Data<Storage>) -> impl Responder {
    let fit_path = storage.fit_file_path(*path);
    match parse_fit_file(&fit_path) {
        Ok(points) => HttpResponse::Ok().json(points),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn run_server(storage: Storage, client: StravaClient) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(storage.clone()))
            .app_data(web::Data::new(client.clone()))
            .service(list_activities)
            .service(list_raw_files)
            .service(download_fit)
            .service(fit_details)
            .service(webhook_verify)
            .service(webhook_event)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
