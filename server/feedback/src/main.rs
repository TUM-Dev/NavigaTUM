use actix_cors::Cors;
use actix_governor::{GlobalKeyExtractor, Governor, GovernorConfigBuilder};
use std::collections::HashMap;

use actix_web::{get, middleware, web, App, HttpResponse, HttpServer};
use actix_web_prometheus::PrometheusMetricsBuilder;

mod core;
mod github;
mod post_feedback;
mod tokens;

const MAX_JSON_PAYLOAD: usize = 1024 * 1024; // 1 MB

#[get("/api/feedback/status")]
async fn health_status_handler() -> HttpResponse {
    let github_link = match std::env::var("GIT_COMMIT_SHA") {
        Ok(hash) => format!("https://github.com/TUM-Dev/navigatum/tree/{hash}"),
        Err(_) => "unknown commit hash, probably running in development".to_string(),
    };
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(format!("healthy\nsource_code: {github_link}"))
}

const SECONDS_PER_DAY: u64 = 60 * 60 * 24;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let feedback_ratelimit = GovernorConfigBuilder::default()
        .key_extractor(GlobalKeyExtractor)
        .per_second(SECONDS_PER_DAY / 100) // replenish new token every .. seconds
        .burst_size(20)
        .finish()
        .expect("Invalid configuration of the governor");

    // metrics
    let labels = HashMap::from([(
        "revision".to_string(),
        std::env::var("GIT_COMMIT_SHA").unwrap_or("development".to_string()),
    )]);
    let prometheus = PrometheusMetricsBuilder::new("navigatum_feedback")
        .endpoint("/metrics")
        .const_labels(labels)
        .build()
        .unwrap();

    let state_feedback = web::Data::new(core::AppStateFeedback::new());
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allowed_methods(vec!["GET", "POST"])
            .max_age(3600);
        App::new()
            .wrap(prometheus.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default().exclude("/api/feedback/status"))
            .wrap(middleware::Compress::default())
            .app_data(web::JsonConfig::default().limit(MAX_JSON_PAYLOAD))
            .service(health_status_handler)
            .app_data(state_feedback.clone())
            .service(post_feedback::send_feedback)
            .service(
                web::scope("/api/feedback/get_token")
                    .wrap(Governor::new(&feedback_ratelimit))
                    .route("", web::post().to(core::get_token)),
            )
    })
    .bind(std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8070".to_string()))?
    .run()
    .await
}
