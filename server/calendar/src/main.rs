mod calendar;
mod models;
mod utils;

use actix_cors::Cors;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};
use std::collections::HashMap;
use std::error::Error;
use log::error;
use structured_logger::async_json::new_writer;
use structured_logger::Builder;

#[derive(Clone, Debug)]
pub struct AppData {
    db: PgPool,
}

const MAX_JSON_PAYLOAD: usize = 1024 * 1024; // 1 MB

#[get("/api/calendar/status")]
async fn health_status_handler(data: web::Data<AppData>) -> HttpResponse {
    let github_link = match std::env::var("GIT_COMMIT_SHA") {
        Ok(hash) => format!("https://github.com/TUM-Dev/navigatum/tree/{hash}"),
        Err(_) => "unknown commit hash, probably running in development".to_string(),
    };
    return match data.db.execute("SELECT 1").await {
        Ok(_) => HttpResponse::Ok()
            .content_type("text/plain")
            .body(format!("healthy\nsource_code: {github_link}")),
        Err(e) => {
            error!("database error: {e:?}",);
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(format!("unhealthy\nsource_code: {github_link}"))
        }
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    Builder::with_level("info")
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();
    let uri = utils::connection_string();
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&uri)
        .await?;

    // metrics
    let labels = HashMap::from([(
        "revision".to_string(),
        std::env::var("GIT_COMMIT_SHA").unwrap_or_else(|_| "development".to_string()),
    )]);
    let prometheus = PrometheusMetricsBuilder::new("navigatum_calendar")
        .endpoint("/metrics")
        .const_labels(labels)
        .build()
        .unwrap();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allowed_methods(vec!["GET"])
            .max_age(3600);

        App::new()
            .wrap(prometheus.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default().exclude("/api/calendar/status"))
            .wrap(middleware::Compress::default())
            .app_data(web::JsonConfig::default().limit(MAX_JSON_PAYLOAD))
            .app_data(web::Data::new(AppData { db: pool.clone() }))
            .service(health_status_handler)
            .service(calendar::calendar_handler)
    })
    .bind(std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3005".to_string()))?
    .run()
    .await?;
    Ok(())
}
