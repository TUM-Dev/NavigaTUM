use std::collections::HashMap;
use std::time::Instant;

use log::info;
use serde_json::Value;

struct DelocalisedValues {
    key: String,
    de: Value,
    en: Value,
}

impl From<HashMap<String, Value>> for DelocalisedValues {
    fn from(value: HashMap<String, Value>) -> Self {
        Self {
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
            key: value
                .clone()
                .get("id")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
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
            r#"INSERT INTO de(key,data)VALUES ($1,$2)"#,
            self.key,
            self.de
        )
        .execute(&mut **tx)
        .await?;

        sqlx::query!(
            r#"INSERT INTO en(key,data)VALUES ($1,$2)"#,
            self.key,
            self.en
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

pub(crate) async fn load_all_to_db(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), crate::BoxedError> {
    let start = Instant::now();
    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let tasks = reqwest::get(format!("{cdn_url}/api_data.json"))
        .await?
        .json::<Vec<HashMap<String, Value>>>()
        .await?
        .into_iter()
        .map(DelocalisedValues::from);
    info!("downloaded data in {elapsed:?}", elapsed = start.elapsed());
    let start = Instant::now();
    for task in tasks {
        task.store(tx).await?;
    }
    info!("loaded data in {elapsed:?}", elapsed = start.elapsed());

    Ok(())
}
