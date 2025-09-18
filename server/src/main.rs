use std::collections::HashMap;
use std::sync::Arc;

use actix_cors::Cors;
use actix_governor::{GlobalKeyExtractor, GovernorConfigBuilder};
use actix_middleware_etag::Etag;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware, web};
use actix_web_prom::{PrometheusMetrics, PrometheusMetricsBuilder};
use meilisearch_sdk::client::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::prelude::*;
use sqlx::{PgPool, Pool, Postgres};
use tokio::sync::{Barrier, RwLock};
use tracing::{debug_span, error, info};
use tracing_actix_web::TracingLogger;

mod docs;
mod limited;
mod localisation;
mod search_executor;
mod setup;
use utoipa_actix_web::{AppExt, scope};
mod db;
pub mod external;
pub mod overlays;
pub mod refresh;
pub mod routes;
use routes::*;

const MAX_JSON_PAYLOAD: usize = 1024 * 1024 * 10; // 10 MB

const SECONDS_PER_DAY: u64 = 60 * 60 * 24;

#[derive(Clone, Debug)]
pub struct AppData {
    /// shared [sqlx::PgPool] to connect to postgis
    pool: PgPool,
    /// necessary, as otherwise we could return empty results during initialisation
    meilisearch_initialised: Arc<RwLock<()>>,
    valhalla: external::valhalla::ValhallaWrapper,
    motis: external::motis::MotisWrapper,
}

impl AppData {
    async fn new() -> Self {
        let pool = PgPoolOptions::new()
            .min_connections(2)
            .connect(&connection_string())
            .await
            .expect("make sure that postgis is running in the background");
        AppData::from(pool)
    }
}
impl From<PgPool> for AppData {
    fn from(pool: PgPool) -> Self {
        AppData {
            pool,
            meilisearch_initialised: Arc::new(Default::default()),
            valhalla: external::valhalla::ValhallaWrapper::default(),
            motis: external::motis::MotisWrapper::default(),
        }
    }
}

/// API healthcheck
///
/// If this endpoint does not return 200, the API is experiencing a catastrophic outage.
/// **Should never happen.**
#[utoipa::path(
    responses(
        (status = 200, description = "API is **healthy**", body = String, content_type = "text/plain", example="healthy\nsource_code: https://github.com/TUM-Dev/navigatum/tree/{hash}"),
        (status = 503, description = "API is **NOT healthy**", body = String, content_type = "text/plain", example="unhealthy\nsource_code: https://github.com/TUM-Dev/navigatum/tree/{hash}"),
    )
)]
#[get("/api/status")]
async fn health_status_handler(data: web::Data<AppData>) -> HttpResponse {
    let github_link = match option_env!("GIT_COMMIT_SHA") {
        Some(hash) => format!("https://github.com/TUM-Dev/navigatum/tree/{hash}"),
        None => "unknown commit hash, probably running in development".to_string(),
    };
    match data.pool.execute("SELECT 1").await {
        Ok(_) => HttpResponse::Ok()
            .content_type("text/plain")
            .body(format!("healthy\nsource_code: {github_link}")),
        Err(e) => {
            error!(error = ?e, "database error");
            HttpResponse::ServiceUnavailable()
                .content_type("text/plain")
                .body(format!("unhealthy\nsource_code: {github_link}"))
        }
    }
}

/// Openapi service definition
///
/// Usefull for consuming in external openapi tooling
#[utoipa::path(
    responses(
        (status = 200, description = "The openapi definition", content_type="application/json")
    )
)]
#[get("/api/openapi.json")]
async fn openapi_doc(openapi: web::Data<utoipa::openapi::OpenApi>) -> impl Responder {
    HttpResponse::Ok().json(openapi)
}

fn connection_string() -> String {
    let username = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
    let password = std::env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "CHANGE_ME".to_string());
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
    let filter = format!(
        "{log_level},hyper=info,rustls=info,h2=info,sqlx=info,hickory_resolver=info,hickory_proto=info"
    );

    let filter = EnvFilter::builder().parse_lossy(filter);

    tracing_log::LogTracer::builder()
        .with_interest_cache(tracing_log::InterestCacheConfig::default())
        .init()
        .expect("the global logger to only be set once");

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(cfg!(not(any(debug_assertions, test))).then(|| Layer::default().json()))
        .with(cfg!(any(debug_assertions, test)).then(|| Layer::default().pretty()));
    tracing::subscriber::set_global_default(registry).unwrap();
}

#[tracing::instrument(skip(pool, meilisearch_initialised, initialisation_started))]
async fn run_maintenance_work(
    pool: Pool<Postgres>,
    meilisearch_initialised: Arc<RwLock<()>>,
    initialisation_started: Arc<Barrier>,
) {
    if std::env::var("SKIP_MS_SETUP") != Ok("true".to_string()) {
        let _ = debug_span!("updating meilisearch data").enter();
        let _ = meilisearch_initialised.write().await;
        initialisation_started.wait().await;
        let ms_url =
            std::env::var("MIELI_URL").unwrap_or_else(|_| "http://localhost:7700".to_string());
        let client = Client::new(ms_url, std::env::var("MEILI_MASTER_KEY").ok()).unwrap();
        setup::meilisearch::setup(&client).await.unwrap();
        setup::meilisearch::load_data(&client).await.unwrap();
    } else {
        info!("skipping the database setup as SKIP_MS_SETUP=true");
        initialisation_started.wait().await;
    }
    if std::env::var("SKIP_DB_SETUP") != Ok("true".to_string()) {
        let _ = debug_span!("updating postgis data").enter();
        setup::database::setup(&pool).await.unwrap();
        setup::database::load_data(&pool).await.unwrap();
        setup::transportation::setup(&pool).await.unwrap();
    } else {
        info!("skipping the database setup as SKIP_DB_SETUP=true");
    }
    let mut set = tokio::task::JoinSet::new();
    let map_pool = pool.clone();
    set.spawn(async move { refresh::indoor_maps::all_entries(&map_pool).await });
    let cal_pool = pool.clone();
    set.spawn(async move { refresh::calendar::all_entries(&cal_pool).await });
    set.join_all().await;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("no provider was set as default beforehand");

    let data = AppData::new().await;

    // without this barrier an external client might race the RWLock for meilisearch_initialised and gain the read lock before it is allowed
    let initialisation_started = Arc::new(Barrier::new(2));
    let maintenance_thread = tokio::spawn(run_maintenance_work(
        data.pool.clone(),
        data.meilisearch_initialised.clone(),
        initialisation_started.clone(),
    ));

    let prometheus = build_metrics();
    let shutdown_pool_clone = data.pool.clone();
    initialisation_started.wait().await;
    // feedback specific initialisation
    let feedback_ratelimit = GovernorConfigBuilder::default()
        .key_extractor(GlobalKeyExtractor)
        .seconds_per_request(SECONDS_PER_DAY / 60) // replenish new token every .. seconds
        .burst_size(100)
        .finish()
        .expect("Invalid configuration of the governor");
    let recorded_tokens = web::Data::new(feedback::tokens::RecordedTokens::default());

    info!("running the server");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allowed_methods(vec!["GET", "POST"])
            .max_age(3600)
            .send_wildcard();

        docs::add_openapi_docs(
            App::new()
                .wrap(Etag::default())
                .wrap(prometheus.clone())
                .wrap(cors)
                .wrap(TracingLogger::default())
                .wrap(middleware::Compress::default())
                .app_data(web::JsonConfig::default().limit(MAX_JSON_PAYLOAD))
                .app_data(web::Data::new(data.clone()))
                .into_utoipa_app()
                .app_data(recorded_tokens.clone())
                .service(health_status_handler)
                .service(calendar::calendar_handler)
                .service(maps::indoor::list_indoor_maps)
                .service(maps::indoor::get_indoor_map)
                .service(maps::route::route_handler)
                .service(search::search_handler)
                .service(locations::details::get_handler)
                .service(locations::nearby::nearby_handler)
                .service(locations::preview::maps_handler)
                .service(feedback::post_feedback::send_feedback)
                .service(feedback::proposed_edits::propose_edits)
                .service(
                    scope("/api/feedback/get_token")
                        .wrap(actix_governor::Governor::new(&feedback_ratelimit))
                        .service(feedback::tokens::get_token),
                )
                .service(openapi_doc),
        )
    })
    .bind(std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3003".to_string()))?
    .run()
    .await?;
    maintenance_thread.abort();
    shutdown_pool_clone.close().await;
    Ok(())
}

#[tracing::instrument]
fn build_metrics() -> PrometheusMetrics {
    let labels = HashMap::from([(
        "revision".to_string(),
        option_env!("GIT_COMMIT_SHA")
            .unwrap_or_else(|| "development")
            .to_string(),
    )]);
    PrometheusMetricsBuilder::new("navigatum_api")
        .endpoint("/api/metrics")
        .const_labels(labels)
        .build()
        .expect("specified metrics are valid")
}
