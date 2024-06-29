use std::collections::HashMap;
use std::time::Instant;

use log::debug;
use serde_json::Value;

pub(super) struct DelocalisedValues {
    key: String,
    hash: i64,
    de: Value,
    en: Value,
}

impl From<HashMap<String, Value>> for DelocalisedValues {
    fn from(value: HashMap<String, Value>) -> Self {
        let key = value
            .get("id")
            .expect("an ID should always exist")
            .as_str()
            .expect("the id should be a valid string")
            .to_string();
        let hash = value
            .get("hash")
            .expect("a hash should always exist")
            .as_i64()
            .expect("a hash should be a valid i64");
        Self {
            key,
            hash,
            de: value
                .clone()
                .into_iter()
                .map(|(k, v)| (k, Self::delocalise(v.clone(), "de")))
                .collect(),
            en: value
                .clone()
                .into_iter()
                .map(|(k, v)| (k, Self::delocalise(v.clone(), "en")))
                .collect(),
        }
    }
}
impl DelocalisedValues {
    fn delocalise(value: Value, language: &'static str) -> Value {
        match value {
            Value::Array(arr) => Value::Array(
                arr.into_iter()
                    .map(|value| Self::delocalise(value, language))
                    .collect(),
            ),
            Value::Object(obj) => {
                if obj.contains_key("de") || obj.contains_key("en") {
                    obj.get(language)
                        .cloned()
                        .unwrap_or(Value::String(String::new()))
                } else {
                    Value::Object(
                        obj.into_iter()
                            .map(|(key, value)| (key, Self::delocalise(value, language)))
                            .filter(|(key, _)| key != "de" && key != "en")
                            .collect(),
                    )
                }
            }
            a => a,
        }
    }
    async fn store(
        self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO de(key,data,hash)
            VALUES ($1,$2,$3)
            ON CONFLICT (key) DO UPDATE
            SET data = EXCLUDED.data,
                hash = EXCLUDED.hash"#,
            self.key,
            self.de,
            self.hash,
        )
        .execute(&mut **tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO en(key,data)
            VALUES ($1,$2)
            ON CONFLICT (key) DO UPDATE
            SET data = EXCLUDED.data"#,
            self.key,
            self.en,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

pub async fn download_updates(
    keys_which_need_updating: &[String],
) -> Result<Vec<DelocalisedValues>, crate::BoxedError> {
    let start = Instant::now();
    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let tasks = reqwest::get(format!("{cdn_url}/api_data.json"))
        .await?
        .json::<Vec<HashMap<String, Value>>>()
        .await?
        .into_iter()
        .map(DelocalisedValues::from)
        .filter(|d| keys_which_need_updating.contains(&d.key))
        .collect::<Vec<DelocalisedValues>>();
    debug!("downloaded data in {elapsed:?}", elapsed = start.elapsed());
    Ok(tasks)
}

pub(super) async fn load_all_to_db(
    tasks: Vec<DelocalisedValues>,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), crate::BoxedError> {
    let start = Instant::now();
    for task in tasks.into_iter() {
        task.store(tx).await?;
    }
    debug!("loaded data in {elapsed:?}", elapsed = start.elapsed());

    Ok(())
}

pub async fn download_status() -> Result<Vec<(String, i64)>, crate::BoxedError> {
    let start = Instant::now();
    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let tasks = reqwest::get(format!("{cdn_url}/status_data.json"))
        .await?
        .json::<Vec<(String, i64)>>()
        .await?;
    debug!(
        "downloaded current status in {elapsed:?}",
        elapsed = start.elapsed()
    );
    Ok(tasks)
}
