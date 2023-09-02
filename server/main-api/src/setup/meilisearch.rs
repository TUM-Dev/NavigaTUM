use log::info;
use meilisearch_sdk::settings::Settings;
use meilisearch_sdk::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

const TIMEOUT: Option<Duration> = Some(Duration::from_secs(20));
const POLLING_RATE: Option<Duration> = Some(Duration::from_millis(50));

#[derive(serde::Deserialize)]
struct Synonyms(HashMap<String, Vec<String>>);

impl Synonyms {
    fn try_load() -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(include_str!("search_synonyms.yaml"))
    }
}

pub(crate) async fn setup_meilisearch() -> Result<(), Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    let ms_url = std::env::var("MIELI_URL").unwrap_or_else(|_| "http://localhost:7700".to_string());
    let client = Client::new(ms_url, std::env::var("MEILI_MASTER_KEY").ok());

    client.health().await.map_err(|e| {
        format!("Meilisearch is not healthy. Please make sure that it is running. error={e:?}")
    })?;

    client
        .create_index("entries", Some("ms_id"))
        .await?
        .wait_for_completion(&client, POLLING_RATE, TIMEOUT)
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
            "exactness",
            "proximity",
            "attribute",
        ])
        .with_searchable_attributes([
            "ms_id",
            "name",
            "arch_name",
            "type",
            "type_common_name",
            "parent_building_names",
            "parent_keywords",
            "address",
            "usage",
        ])
        .with_synonyms(Synonyms::try_load()?.0);

    entries
        .set_settings(&settings)
        .await?
        .wait_for_completion(&client, POLLING_RATE, TIMEOUT)
        .await?;

    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let documents = reqwest::get(format!("{cdn_url}/search_data.json"))
        .await?
        .json::<Vec<Value>>()
        .await?;
    entries
        .add_documents(&documents, Some("ms_id"))
        .await?
        .wait_for_completion(&client, POLLING_RATE, TIMEOUT)
        .await?;
    info!(
        "{cnt} documents added in {elapsed:?}",
        elapsed = start.elapsed(),
        cnt = documents.len()
    );
    Ok(())
}
