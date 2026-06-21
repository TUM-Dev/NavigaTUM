use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use actix_cors::Cors;
use actix_governor::{GlobalKeyExtractor, GovernorConfigBuilder};
use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware, web};
use actix_web_prom::{PrometheusMetrics, PrometheusMetricsBuilder};
use meilisearch_sdk::client::Client;
use moka::future::Cache;
use rustls::crypto::aws_lc_rs;
use sqlx::postgres::PgPoolOptions;
use sqlx::prelude::*;
use sqlx::{PgPool, Pool, Postgres};
use tokio::sync::{Barrier, RwLock};
use tokio::task::JoinSet;
use tracing::{Instrument as _, debug_span, error, info, subscriber};
use tracing_actix_web::TracingLogger;
use utoipa::openapi::OpenApi;

mod docs;
mod limited;
mod localisation;
mod search_executor;
mod setup;
use utoipa_actix_web::{AppExt as _, scope};
mod db;
pub mod external;
pub mod overlays;
pub mod refresh;
pub mod routes;
use routes::{calendar, feedback, locations, maps, mensa, search};

const MAX_JSON_PAYLOAD: usize = 1024 * 1024 * 10; // 10 MB

const SECONDS_PER_DAY: u64 = 60 * 60 * 24;

#[derive(Clone, Debug)]
pub struct AppData {
    /// shared [`sqlx::PgPool`] to connect to postgis
    pool: PgPool,
    /// necessary, as otherwise we could return empty results during initialisation
    meilisearch_initialised: Arc<RwLock<()>>,
    valhalla: external::valhalla::ValhallaWrapper,
    motis: external::motis::MotisWrapper,
    /// moka cache for search results (size ~= 0.1Mi per entry)
    search_cache: Cache<search::SearchCacheKey, Vec<search_executor::ResultsSection>>,
}

impl AppData {
    async fn new() -> Self {
        // max bumped to fit 11 parallel post-load_data loaders (9 derived
        // tables + transportation + the tumonline_orgs->events chain),
        // plus headroom for request handling while setup runs.
        let pool = PgPoolOptions::new()
            .min_connections(2)
            .max_connections(20)
            .connect(&connection_string())
            .await
            .expect("make sure that postgis is running in the background");
        Self::from(pool)
    }
}
impl From<PgPool> for AppData {
    fn from(pool: PgPool) -> Self {
        Self {
            pool,
            meilisearch_initialised: Arc::new(RwLock::default()),
            valhalla: external::valhalla::ValhallaWrapper::default(),
            motis: external::motis::MotisWrapper::default(),
            search_cache: Cache::builder().max_capacity(200).build(),
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
#[get("/api/status", wrap = "actix_middleware_etag::Etag::default()")]
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
#[get("/api/openapi.json", wrap = "actix_middleware_etag::Etag::default()")]
async fn openapi_doc(openapi: web::Data<OpenApi>) -> impl Responder {
    HttpResponse::Ok().json(openapi)
}

fn connection_string() -> String {
    let username = env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
    let password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "CHANGE_ME".to_string());
    let url = env::var("POSTGRES_URL").unwrap_or_else(|_| "localhost".to_string());
    let db = env::var("POSTGRES_DB").unwrap_or_else(|_| username.clone());
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
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| default_level.to_string());
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
    subscriber::set_global_default(registry)
        .expect("the tracing subscriber to be set as the global default");
}

#[tracing::instrument(skip(
    pool,
    meilisearch_initialised,
    initialisation_started,
    repo_pool,
    calendar_metrics
))]
async fn run_maintenance_work(
    pool: Pool<Postgres>,
    meilisearch_initialised: Arc<RwLock<()>>,
    initialisation_started: Arc<Barrier>,
    repo_pool: Arc<feedback::proposed_edits::repo_pool::RepoPool>,
    calendar_metrics: refresh::calendar::CalendarMetrics,
) {
    let meilisearch_enabled = env::var("SKIP_MS_SETUP") != Ok("true".to_string());
    if meilisearch_enabled {
        async {
            // Hold the write lock across setup so request handlers block on the read lock
            // until meilisearch is populated. The barrier below guarantees the write lock is
            // taken before `main` starts serving, closing the race the barrier exists to prevent.
            let _meilisearch_guard = meilisearch_initialised.write().await;
            initialisation_started.wait().await;
            let ms_url =
                env::var("MIELI_URL").unwrap_or_else(|_| "http://localhost:7700".to_string());
            let client = Client::new(ms_url, env::var("MEILI_MASTER_KEY").ok())
                .expect("a valid meilisearch client");
            setup::meilisearch::setup(&client)
                .await
                .expect("meilisearch setup to succeed");
            setup::meilisearch::load_data(&client)
                .await
                .expect("meilisearch initial data load to succeed");
        }
        .instrument(debug_span!("updating meilisearch data"))
        .await;
    } else {
        info!("skipping the meilisearch setup as SKIP_MS_SETUP=true");
        initialisation_started.wait().await;
    }
    if env::var("SKIP_DB_SETUP") == Ok("true".to_string()) {
        info!("skipping the database setup as SKIP_DB_SETUP=true");
    } else {
        async {
            setup::database::setup(&pool)
                .await
                .expect("postgis schema setup to succeed");
            setup::database::load_data(&pool)
                .await
                .expect("postgis initial data load to succeed");
            // Once `de`/`en` are populated, every remaining loader fans out.
            // The lookup tables FK back to `de`/`en` only, transportation is
            // FK-isolated, and tumonline_orgs -> events is a self-contained
            // sequential pair (events.organising_org_id REFERENCES
            // tumonline_orgs.org_id).
            let mut loaders = JoinSet::new();
            loaders.spawn(setup::transportation::setup(pool.clone()));
            {
                let p = pool.clone();
                loaders.spawn(async move {
                    setup::tumonline_orgs::setup(p.clone()).await?;
                    setup::events::setup(p).await
                });
            }
            loaders.spawn(setup::ranking_factors::setup(pool.clone()));
            loaders.spawn(setup::operators_de::setup(pool.clone()));
            loaders.spawn(setup::operators_en::setup(pool.clone()));
            loaders.spawn(setup::sources::setup(pool.clone()));
            loaders.spawn(setup::usages::setup(pool.clone()));
            loaders.spawn(setup::urls_de::setup(pool.clone()));
            loaders.spawn(setup::urls_en::setup(pool.clone()));
            loaders.spawn(setup::parents::setup(pool.clone()));
            loaders.spawn(setup::location_images::setup(pool.clone()));
            while let Some(res) = loaders.join_next().await {
                res.expect("loader task to complete")
                    .expect("loader setup to succeed");
            }
        }
        .instrument(debug_span!("updating postgis data"))
        .await;
    }
    let mut set = JoinSet::new();
    let cal_pool = pool.clone();
    let scrape_metrics = calendar_metrics.clone();
    set.spawn(async move { refresh::calendar::all_entries(&cal_pool, scrape_metrics).await });
    let freshness_pool = pool.clone();
    set.spawn(async move {
        refresh::calendar::record_freshness(&freshness_pool, calendar_metrics).await
    });
    // The lecture facet is derived from the (continuously scraped) calendar, so
    // it only makes sense when Meilisearch is the destination. It builds its own
    // client because the setup client above is scoped to initial loading.
    if meilisearch_enabled {
        let lecture_pool = pool.clone();
        let ms_url = env::var("MIELI_URL").unwrap_or_else(|_| "http://localhost:7700".to_string());
        let lecture_client = Client::new(ms_url, env::var("MEILI_MASTER_KEY").ok())
            .expect("a valid meilisearch client");
        set.spawn(async move {
            refresh::lectures::refresh_lectures(lecture_pool, lecture_client).await;
        });
    }
    set.join_all().await;

    // Warm up the bare repo for edit proposals after all other setup is done.
    repo_pool.warm().await;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();
    aws_lc_rs::default_provider()
        .install_default()
        .expect("no provider was set as default beforehand");

    let data = AppData::new().await;

    // Persistent bare repo for edit proposals - the initial clone runs in the
    // background after MS/DB setup; requests before that fall back to lazy init.
    let repo_pool = Arc::new(feedback::proposed_edits::repo_pool::RepoPool::new());

    // without this barrier an external client might race the RWLock for meilisearch_initialised and gain the read lock before it is allowed
    let initialisation_started = Arc::new(Barrier::new(2));
    let prometheus = build_metrics();
    let calendar_metrics = refresh::calendar::CalendarMetrics::new(&prometheus.registry)
        .expect("calendar metrics to register with the prometheus registry");
    let maintenance_thread = tokio::spawn(run_maintenance_work(
        data.pool.clone(),
        Arc::clone(&data.meilisearch_initialised),
        Arc::clone(&initialisation_started),
        Arc::clone(&repo_pool),
        calendar_metrics,
    ));

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
    let repo_pool = web::Data::new(repo_pool);
    let eat_api_menus = web::Data::new(mensa::EatApiMenus::default());

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
                .wrap(prometheus.clone())
                .wrap(cors)
                .wrap(TracingLogger::default())
                .wrap(middleware::Compress::default())
                .app_data(web::JsonConfig::default().limit(MAX_JSON_PAYLOAD))
                .app_data(web::Data::new(data.clone()))
                .into_utoipa_app()
                .app_data(recorded_tokens.clone())
                .app_data(repo_pool.clone())
                .app_data(eat_api_menus.clone())
                .service(health_status_handler)
                .service(calendar::calendar_handler)
                .service(maps::route::route_handler)
                .service(mensa::menu_handler)
                .service(search::search_handler)
                .service(locations::details::get_handler)
                .service(locations::nearby::nearby_handler)
                .service(locations::preview::maps_handler)
                .service(locations::qr_code::qr_code_handler)
                .service(feedback::post_feedback::send_feedback)
                .service(feedback::proposed_edits::propose_edits)
                .service(
                    scope("/api/feedback/get_token")
                        .wrap(actix_governor::Governor::new(&feedback_ratelimit))
                        .service(feedback::tokens::get_token),
                )
                .service(openapi_doc)
                .map(|app| {
                    // Add static file serving outside utoipa to avoid trait bound requirements
                    // Note: use_last_modified and use_etag is disabled to prevent actix_files from generating
                    // ETags with invalid format (containing colons). The ETag middleware above
                    // will generate RFC-compliant ETags for all responses including static files.
                    // (this would be a runtime panic)
                    app.service(
                        actix_files::Files::new("/cdn", "/cdn")
                            .show_files_listing()
                            .redirect_to_slash_directory()
                            .with_permanent_redirect()
                            .prefer_utf8(true),
                    )
                }),
        )
    })
    .bind(env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3003".to_string()))?
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
