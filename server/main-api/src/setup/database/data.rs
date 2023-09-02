use log::info;
use serde_json::Value;
use sqlx::SqlitePool;
use std::collections::HashMap;

struct DelocalisedValue {
    de: Value,
    en: Value,
    key: String,
}

impl From<(String, Value)> for DelocalisedValue {
    fn from((key, value): (String, Value)) -> Self {
        Self {
            key,
            de: value.clone(),
            en: value.clone(),
        }
    }
}

pub(crate) async fn load_all_to_db(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let raw_data = reqwest::get(format!("{cdn_url}/api_data.json"))
        .await?
        .json::<HashMap<String, Value>>()
        .await?;
    let tasks = raw_data
        .into_iter()
        .map(DelocalisedValue::from)
        .map(|delocalised| delocalised.store(pool));
    futures::future::try_join_all(tasks).await?;
    info!("loaded data");

    Ok(())
}
