use meilisearch_sdk::client::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};

#[cfg(not(feature = "skip_db_setup"))]
pub struct PostgresTestContainer {
    _container: ContainerAsync<postgres::Postgres>,
    pub pool: Pool<Postgres>,
}

#[cfg(not(feature = "skip_db_setup"))]
impl PostgresTestContainer {
    /// Create a postgres instance for testing against
    pub async fn new() -> Self {
        let container = postgres::Postgres::default()
            .with_tag("16")
            .start()
            .await
            .unwrap();
        let connection_string = format!(
            "postgres://postgres:postgres@{host}:{port}/postgres",
            host = container.get_host().await.unwrap(),
            port = container.get_host_port_ipv4(5432).await.unwrap(),
        );
        let pool = PgPoolOptions::new()
            .connect(&connection_string)
            .await
            .unwrap();
        crate::setup::database::setup(&pool).await.unwrap();
        Self {
            _container: container,
            pool,
        }
    }
}

#[cfg(not(feature = "skip_ms_setup"))]
pub struct MeiliSearchTestContainer {
    _container: ContainerAsync<GenericImage>,
    pub client: Client,
}

#[cfg(not(feature = "skip_ms_setup"))]
impl MeiliSearchTestContainer {
    /// Create a meilisearch instance for testing against
    pub async fn new() -> Self {
        let container = GenericImage::new("getmeili/meilisearch", "v1.8.0")
            .with_exposed_port(ContainerPort::Tcp(7700))
            .with_wait_for(WaitFor::message_on_stderr(
                "Actix runtime found; starting in Actix runtime",
            ))
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
}

#[tokio::test]
#[cfg(not(feature = "skip_db_setup"))]
async fn test_db_setup() {
    crate::setup_logging();
    let pg = PostgresTestContainer::new().await;
    crate::setup::database::load_data(&pg.pool).await.unwrap();
}

#[tokio::test]
#[cfg(not(feature = "skip_ms_setup"))]
async fn test_meilisearch_setup() {
    crate::setup_logging();
    let ms = MeiliSearchTestContainer::new().await;
    crate::setup::meilisearch::load_data(&ms.client)
        .await
        .unwrap();
}
