use std::sync::Arc;

use moka::future::Cache;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::RwLock;

use crate::external;
use crate::routes::search;
use crate::search_executor;

#[derive(Clone, Debug)]
pub struct AppData {
    /// shared [sqlx::PgPool] to connect to postgis
    pub pool: PgPool,
    /// necessary, as otherwise we could return empty results during initialisation
    pub meilisearch_initialised: Arc<RwLock<()>>,
    pub valhalla: external::valhalla::ValhallaWrapper,
    pub motis: external::motis::MotisWrapper,
    /// moka cache for search results (size ~= 0.1Mi per entry)
    pub search_cache: Cache<search::SearchCacheKey, Vec<search_executor::ResultsSection>>,
}

impl AppData {
    pub async fn new() -> Self {
        let username = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
        let password = std::env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "CHANGE_ME".to_string());
        let url = std::env::var("POSTGRES_URL").unwrap_or_else(|_| "localhost".to_string());
        let db = std::env::var("POSTGRES_DB").unwrap_or_else(|_| username.clone());
        let connection_string = format!("postgres://{username}:{password}@{url}/{db}");
        
        let pool = PgPoolOptions::new()
            .min_connections(2)
            .connect(&connection_string)
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
            search_cache: Cache::builder().max_capacity(200).build(),
        }
    }
}
