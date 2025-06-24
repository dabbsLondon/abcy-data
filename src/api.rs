use actix_web::{get, web, App, HttpServer, Responder};
use serde::Serialize;
use crate::storage::Storage;
use std::fs;
use std::ffi::OsStr;

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
    if let Ok(entries) = fs::read_dir(&storage.data_dir) {
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

pub async fn run_server(storage: Storage) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(storage.clone()))
            .service(list_activities)
            .service(list_raw_files)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
