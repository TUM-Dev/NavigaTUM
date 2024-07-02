use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;

use actix_cors::Cors;
use actix_web::{App, get, HttpResponse, HttpServer, middleware, web};
use actix_web_prom::PrometheusMetricsBuilder;
use meilisearch_sdk::client::Client;
use sentry::SessionMode;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use sqlx::prelude::*;
use tracing::{debug, debug_span, error, info};
use tracing_actix_web::TracingLogger;

mod calendar;
mod details;
mod feedback;
mod limited;
mod maps;
mod models;
mod search;
mod setup;
mod utils;

type BoxedError = Box<dyn Error + Send + Sync>;

const MAX_JSON_PAYLOAD: usize = 1024 * 1024; // 1 MB

#[derive(Debug)]
pub struct AppData {
    db: PgPool,
}

#[get("/api/status")]
async fn health_status_handler(data: web::Data<AppData>) -> HttpResponse {
    let github_link = match option_env!("GIT_COMMIT_SHA") {
        Some(hash) => format!("https://github.com/TUM-Dev/navigatum/tree/{hash}"),
        None => "unknown commit hash, probably running in development".to_string(),
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

fn connection_string() -> String {
    let username = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
    let password = std::env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "password".to_string());
    let url = std::env::var("POSTGRES_URL").unwrap_or_else(|_| "localhost".to_string());
    let db = std::env::var("POSTGRES_DB").unwrap_or_else(|_| username.clone());
    format!("postgres://{username}:{password}@{url}/{db}")
}

pub fn setup_logging() {
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::fmt::Layer;
    use tracing_subscriber::prelude::*;
    let default_level = if cfg!(any(debug_assertions, test)) {
        "debug"
    } else {
        "info"
    };
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| default_level.to_string());
    // these overrides exist to filter away stuff I don't think we should ever care about
    let filter = format!("{log_level},hyper=info,rustls=info,h2=info,sqlx=info,hickory_resolver=info,hickory_proto=info");

    let filter = EnvFilter::builder().parse_lossy(filter);

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(sentry::integrations::tracing::layer())
        .with(cfg!(not(any(debug_assertions, test))).then(|| Layer::default().json()))
        .with(cfg!(any(debug_assertions, test)).then(|| Layer::default().pretty()));
    tracing::subscriber::set_global_default(registry).unwrap();
}

fn main() -> Result<(), BoxedError> {
    setup_logging();
    let release = match option_env!("GIT_COMMIT_SHA") {
        Some(s) => Some(Cow::Borrowed(s)),
        None => sentry::release_name!(),
    };
    let _guard = sentry::init((
        "https://8f2054d6294447a1b573ea4badb76778@sentry.mm.rbg.tum.de/8",
        sentry::ClientOptions {
            release,
            traces_sample_rate: 1.0,
            session_mode: SessionMode::Request,
            auto_session_tracking: true,
            ..Default::default()
        },
    ));
    std::env::set_var("RUST_BACKTRACE", "1");

    actix_web::rt::System::new().block_on(async { run().await })?;
    Ok(())
}
async fn run_maintenance_work() {
    let pool = PgPoolOptions::new()
        .connect(&connection_string())
        .await
        .expect("make sure that postgres is running in the background");
    if std::env::var("SKIP_MS_SETUP") != Ok("true".to_string()) {
        let _ = debug_span!("updating meilisearch data").enter();
        let ms_url =
            std::env::var("MIELI_URL").unwrap_or_else(|_| "http://localhost:7700".to_string());
        let client = Client::new(ms_url, std::env::var("MEILI_MASTER_KEY").ok()).unwrap();
        setup::meilisearch::setup(&client).await.unwrap();
        setup::meilisearch::load_data(&client).await.unwrap();
    } else {
        info!("skipping the database setup as SKIP_MS_SETUP=true");
    }
    if std::env::var("SKIP_DB_SETUP") != Ok("true".to_string()) {
        let _ = debug_span!("updating postgres data").enter();
        setup::database::setup(&pool).await.unwrap();
        setup::database::load_data(&pool).await.unwrap();
    } else {
        info!("skipping the database setup as SKIP_DB_SETUP=true");
    }
    calendar::refresh::all_entries(&pool).await;
}

/// we split main and run because otherwise sentry could not be properly instrumented
async fn run() -> Result<(), BoxedError> {
    let maintenance_thread = tokio::spawn(run_maintenance_work());

    debug!("setting up metrics");
    let labels = HashMap::from([(
        "revision".to_string(),
        option_env!("GIT_COMMIT_SHA")
            .unwrap_or_else(|| "development")
            .to_string(),
    )]);
    let prometheus = PrometheusMetricsBuilder::new("navigatum_mainapi")
        .endpoint("/api/main/metrics")
        .const_labels(labels)
        .build()
        .unwrap();
    let pool = PgPoolOptions::new()
        .connect(&connection_string())
        .await
        .expect("make sure that postgres is running in the background");
    let shutdown_pool_clone = pool.clone();
    info!("running the server");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allowed_methods(vec!["GET", "POST"])
            .max_age(3600);

        App::new()
            .wrap(prometheus.clone())
            .wrap(cors)
            .wrap(TracingLogger::default())
            .wrap(middleware::Compress::default())
            .wrap(sentry_actix::Sentry::new())
            .app_data(web::JsonConfig::default().limit(MAX_JSON_PAYLOAD))
            .app_data(web::Data::new(AppData { db: pool.clone() }))
            .service(health_status_handler)
            .service(calendar::calendar_handler)
            .service(web::scope("/api/preview").configure(maps::configure))
            .service(web::scope("/api/feedback").configure(feedback::configure))
            .service(details::get_handler)
            .service(search::search_handler)
    })
    .bind(std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3003".to_string()))?
    .run()
    .await?;
    maintenance_thread.abort();
    shutdown_pool_clone.close().await;
    Ok(())
}
