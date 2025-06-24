use actix_web::{get, web, App, HttpServer, Responder};
use serde::Serialize;
use crate::storage::Storage;
use std::fs;

#[derive(Serialize)]
struct ApiActivity {
    id: u64,
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

pub async fn run_server(storage: Storage) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(storage.clone()))
            .service(list_activities)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
