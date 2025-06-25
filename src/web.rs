use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder};
use crate::auth::Auth;
use crate::fetch;
use crate::storage::Storage;
use crate::utils::Config;

#[get("/activities")]
async fn activities(storage: web::Data<Storage>) -> impl Responder {
    match storage.list_activities().await {
        Ok(a) => HttpResponse::Ok().json(a),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/activity/{id}")]
async fn activity(id: web::Path<u64>, storage: web::Data<Storage>) -> impl Responder {
    match storage.load_activity(*id).await {
        Ok(d) => HttpResponse::Ok().json(d),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[get("/files")]
async fn files(storage: web::Data<Storage>) -> impl Responder {
    match storage.list_files().await {
        Ok(f) => HttpResponse::Ok().json(f),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(serde::Deserialize)]
struct WebhookEvent {
    object_type: String,
    aspect_type: String,
}

#[post("/webhook")]
async fn webhook(event: web::Json<WebhookEvent>, auth: web::Data<Auth>, storage: web::Data<Storage>) -> impl Responder {
    if event.object_type == "activity" && event.aspect_type == "create" {
        let auth = auth.clone();
        let storage = storage.clone();
        actix_web::rt::spawn(async move {
            let _ = fetch::download_latest(&auth, &storage, 1).await;
        });
    }
    HttpResponse::Ok()
}

pub async fn run(_config: Config, auth: Auth, storage: Storage) -> std::io::Result<()> {
    let data_auth = web::Data::new(auth);
    let data_storage = web::Data::new(storage);
    HttpServer::new(move || {
        App::new()
            .app_data(data_auth.clone())
            .app_data(data_storage.clone())
            .service(activities)
            .service(activity)
            .service(files)
            .service(webhook)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
