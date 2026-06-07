#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::absolute_paths,
    reason = "test fixtures (testcontainer setup) consumed only from #[cfg(test)] blocks; production-code lints are relaxed for the same reasons as in per-file `mod tests` blocks"
)]

use meilisearch_sdk::client::Client;
use testcontainers::{ContainerAsync, ImageExt as _};
use testcontainers_modules::{meilisearch, testcontainers::runners::AsyncRunner as _};
use tracing::{error, info};

pub struct PostgresTestContainer {
    _container: ContainerAsync<testcontainers_modules::postgres::Postgres>,
    pub pool: sqlx::Pool<sqlx::Postgres>,
}

impl PostgresTestContainer {
    /// Create a postgres instance for testing against
    pub async fn new() -> Self {
        let container = testcontainers_modules::postgres::Postgres::default()
            .with_tag("18-3.6")
            .with_name("postgis/postgis")
            .start()
            .await
            .unwrap();
        let connection_string = format!(
            "postgres://postgres:postgres@{host}:{port}/postgres",
            host = container.get_host().await.unwrap(),
            port = container.get_host_port_ipv4(5432).await.unwrap(),
        );
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect(&connection_string)
            .await
            .unwrap();
        crate::setup::database::setup(&pool).await.unwrap();
        Self {
            _container: container,
            pool,
        }
    }
    pub async fn load_data_retrying(&self) {
        // Retry up to 10 times with 2-second delays.
        // Since download operations already have their own retry logic,
        // we don't need as many outer retries to avoid excessive wait times.
        for i in 0..10 {
            let res = crate::setup::database::load_data(&self.pool).await;
            if let Err(e) = res {
                error!(error = ?e, try_num = i, "failed to load db. Retrying up to 10 times");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            } else {
                info!("successfully initalised the db in try {i}");
                return;
            }
        }

        panic!("could not initialise db after 10 retries")
    }
}

pub struct MeiliSearchTestContainer {
    _container: ContainerAsync<meilisearch::Meilisearch>,
    pub client: Client,
}

impl MeiliSearchTestContainer {
    /// Create a meilisearch instance for testing against
    pub async fn new() -> Self {
        let container = meilisearch::Meilisearch::default()
            .with_tag("v1.29.0")
            .start()
            .await
            .unwrap();
        let meili_url = format!(
            "http://{host}:{port}",
            host = container.get_host().await.unwrap(),
            port = container.get_host_port_ipv4(7700).await.unwrap(),
        );

        let client = Client::new(meili_url.clone(), None::<String>).unwrap();
        super::meilisearch::setup(&client).await.unwrap();
        Self {
            _container: container,
            client,
        }
    }

    pub async fn load_data_retrying(&self) {
        // Retry up to 10 times with 2-second delays.
        // Since download_file already has 5 retries with exponential backoff,
        // we don't need as many outer retries to avoid excessive wait times.
        for i in 0..10 {
            let res = crate::setup::meilisearch::load_data(&self.client).await;
            if let Err(e) = res {
                error!(error = ?e, try_num = i, "failed to load meilisearch data. Retrying up to 10 times");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            } else {
                info!("successfully loaded meilisearch data in try {i}");
                return;
            }
        }

        panic!("could not load meilisearch data after 10 retries")
    }
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_db_setup() {
    let pg = PostgresTestContainer::new().await;
    pg.load_data_retrying().await;
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_meilisearch_setup() {
    let ms = MeiliSearchTestContainer::new().await;
    ms.load_data_retrying().await;
}

/// Inserts a row into `de` with the minimal JSON needed to satisfy the
/// generated NOT NULL columns (`name`, `type`, `type_common_name`, `coords.lat`, `coords.lon`).
async fn seed_de(pool: &sqlx::Pool<sqlx::Postgres>, key: &str, hash: i64) {
    let data = serde_json::json!({
        "name": key,
        "type": "room",
        "type_common_name": "room",
        "coords": { "lat": 0.0, "lon": 0.0, "source": "test" },
    });
    sqlx::query!(
        "INSERT INTO de(key, data, hash) VALUES ($1, $2, $3)",
        key,
        data,
        hash,
    )
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
#[tracing_test::traced_test]
async fn find_keys_returns_missing_and_changed_but_not_unchanged() {
    use crate::limited::vec::LimitedVec;

    let pg = PostgresTestContainer::new().await;
    seed_de(&pg.pool, "A", 1).await;
    seed_de(&pg.pool, "B", 2).await;

    let keys = LimitedVec(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    let hashes = LimitedVec(vec![1, 99, 7]);

    let result = crate::setup::database::find_keys_which_need_updating(&pg.pool, &keys, &hashes)
        .await
        .unwrap();
    let mut got = result.0;
    got.sort();
    assert_eq!(got, vec!["B".to_string(), "C".to_string()]);
}

#[tokio::test]
#[tracing_test::traced_test]
async fn find_keys_returns_all_when_db_is_empty() {
    use crate::limited::vec::LimitedVec;

    let pg = PostgresTestContainer::new().await;

    let keys = LimitedVec(vec!["A".to_string(), "B".to_string()]);
    let hashes = LimitedVec(vec![1, 2]);

    let result = crate::setup::database::find_keys_which_need_updating(&pg.pool, &keys, &hashes)
        .await
        .unwrap();
    let mut got = result.0;
    got.sort();
    assert_eq!(got, vec!["A".to_string(), "B".to_string()]);
}
