use actix_cors::Cors;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer};

mod core;
mod maps;
mod models;
mod schema;
mod utils;

const MAX_JSON_PAYLOAD: usize = 1024 * 1024; // 1 MB

#[get("/api/status")]
async fn health_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain")
        .body("healthy")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allowed_methods(vec!["GET"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default().exclude("/api/status"))
            .wrap(middleware::Compress::default())
            .app_data(web::JsonConfig::default().limit(MAX_JSON_PAYLOAD))
            .service(health_handler)
            .service(web::scope("/api/preview").configure(maps::configure))
            .service(web::scope("/api").configure(core::configure))
    })
    .bind(std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string()))?
    .run()
    .await
}
