use std::collections::HashMap;
use std::env;
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::limited::vec::LimitedVec;
use crate::setup::file_loader;
use bytes::Bytes;
use parquet::file::reader::{FileReader as _, SerializedFileReader};
use parquet::record::Field;
use serde_json::Value;
use sqlx::{Postgres, Transaction};

#[derive(Clone)]
pub(super) struct DelocalisedValues {
    key: String,
    hash: i64,
    de: Value,
    en: Value,
}
#[expect(
    clippy::missing_fields_in_debug,
    reason = "Debug intentionally elides the de/en JSON payloads for log readability"
)]
impl fmt::Debug for DelocalisedValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DelocalisedValues")
            .field("key", &self.key)
            .field("hash", &self.hash)
            .finish()
    }
}

impl PartialEq<Self> for DelocalisedValues {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}
impl Eq for DelocalisedValues {}

impl Hash for DelocalisedValues {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i64(self.hash);
    }
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
    async fn store(self, tx: &mut Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
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
#[tracing::instrument]
pub async fn download_updates(
    keys_which_need_updating: &LimitedVec<String>,
) -> anyhow::Result<LimitedVec<DelocalisedValues>> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let tasks = file_loader::load_json_or_download::<Vec<HashMap<String, Value>>>(
        "api_data.json",
        &cdn_url,
    )
    .await?
    .into_iter()
    .map(DelocalisedValues::from)
    .filter(|d| keys_which_need_updating.0.contains(&d.key))
    .collect::<LimitedVec<DelocalisedValues>>();
    Ok(tasks)
}
#[tracing::instrument(skip(tx))]
pub(super) async fn load_all_to_db(
    tasks: LimitedVec<DelocalisedValues>,
    tx: &mut Transaction<'_, Postgres>,
) -> anyhow::Result<()> {
    for task in tasks {
        task.store(tx).await?;
    }
    Ok(())
}
#[tracing::instrument]
pub async fn download_status() -> anyhow::Result<(LimitedVec<String>, LimitedVec<i64>)> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("status_data.parquet", &cdn_url).await?;
    let reader = SerializedFileReader::new(Bytes::from(body))?;
    let mut id_col: Vec<String> = Vec::new();
    let mut hash_col: Vec<i64> = Vec::new();
    for row in reader.get_row_iter(None)? {
        let row = row?;
        for (col_name, field) in row.get_column_iter() {
            match (col_name.as_str(), field) {
                ("id", Field::Str(v)) => id_col.push(v.clone()),
                ("hash", Field::Long(v)) => hash_col.push(*v),
                _ => {}
            }
        }
    }
    Ok((LimitedVec(id_col), LimitedVec(hash_col)))
}
