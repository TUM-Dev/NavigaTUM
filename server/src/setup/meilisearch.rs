use std::collections::HashMap;
use std::time::Duration;

use meilisearch_sdk::client::Client;
use meilisearch_sdk::settings::Settings;
use meilisearch_sdk::tasks::Task;
use serde_json::Value;
use tracing::{debug, error, info};

use crate::setup::file_loader;

const TIMEOUT: Option<Duration> = Some(Duration::from_secs(60));
const POLLING_RATE: Option<Duration> = Some(Duration::from_millis(250));

#[derive(serde::Deserialize)]
struct Synonyms(HashMap<String, Vec<String>>);

impl Synonyms {
    fn try_load() -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(include_str!("search_synonyms.yaml"))
    }
}
#[tracing::instrument(skip(client))]
async fn wait_for_healthy(client: &Client) {
    let mut counter = 0;
    loop {
        match client.health().await {
            Ok(status) => {
                if status.status == "available" {
                    return;
                } else if counter > 10 {
                    error!(
                        status = status.status,
                        "Meilisearch responding, but is not available. Please make sure that it is running"
                    );
                }
            }
            Err(e) => {
                if counter > 10 {
                    error!(
                        error = ?e,
                        "Meilisearch unhealthy. Please make sure that it is running",
                    );
                }
            }
        }
        counter += 1;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
#[tracing::instrument(skip(client))]
pub async fn setup(client: &Client) -> anyhow::Result<()> {
    debug!("waiting for Meilisearch to be healthy");
    wait_for_healthy(client).await;
    info!("Meilisearch is healthy");

    client
        .create_index("entries", Some("ms_id"))
        .await?
        .wait_for_completion(client, POLLING_RATE, TIMEOUT)
        .await?;
    let entries = client.index("entries");

    let settings = Settings::new()
        .with_filterable_attributes([
            "facet",
            "parent_keywords",
            "parent_building_names",
            "campus",
            "type",
            "usage",
        ])
        .with_ranking_rules([
            "words",
            "typo",
            "rank:desc",
            "proximity",
            "attribute",
            "sort",
            "exactness",
        ])
        .with_sortable_attributes(["_geo"])
        .with_searchable_attributes([
            "room_code",
            "room_code_normalised",
            "name",
            "arch_name",
            "arch_name_normalised",
            "type",
            "type_common_name",
            "parent_building_names",
            "parent_keywords",
            "usage",
            "address",
            "operator_name",
        ])
        .with_synonyms(Synonyms::try_load()?.0);

    let res = entries
        .set_settings(&settings)
        .await?
        .wait_for_completion(client, POLLING_RATE, TIMEOUT)
        .await?;
    if let Task::Failed { content } = res {
        panic!("Failed to add settings to Meilisearch: {content:?}");
    }
    Ok(())
}
#[tracing::instrument(skip(client))]
pub async fn load_data(client: &Client) -> anyhow::Result<()> {
    let entries = client.index("entries");
    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let documents =
        file_loader::load_json_or_download::<Vec<Value>>("search_data.json", &cdn_url).await?;
    let res = entries
        .add_documents(&documents, Some("ms_id"))
        .await?
        .wait_for_completion(client, POLLING_RATE, TIMEOUT)
        .await?;
    if let Task::Failed { content } = res {
        panic!("Failed to add documents to Meilisearch: {content:?}");
    }

    info!("{cnt} documents added", cnt = documents.len());
    Ok(())
}
