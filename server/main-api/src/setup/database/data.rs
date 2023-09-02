use log::info;
use serde_json::Value;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::time::Instant;

struct ExtractedFields {
    name: String,
    tumonline_room_nr: Option<i32>,
    r#type: String,
    type_common_name: String,
    lat: f32,
    lon: f32,
}
impl ExtractedFields {
    fn extract(value: Value) -> Option<Self> {
        let obj = value.as_object()?;
        let props = obj.get("props")?.as_object()?;
        let tumonline_room_nr = match props.get("tumonline_room_nr") {
            Some(v) => Some(v.as_i64()? as i32),
            None => None,
        };
        let coords = obj.get("props")?.as_object()?;
        Some(ExtractedFields {
            name: obj.get("name")?.as_str()?.to_string(),
            tumonline_room_nr,
            r#type: obj.get("type")?.as_str()?.to_string(),
            type_common_name: obj.get("type_common_name")?.as_str()?.to_string(),
            lat: coords.get("lat")?.as_f64().unwrap_or(48.14903) as f32,
            lon: coords.get("lon")?.as_f64().unwrap_or(11.56735) as f32,
        })
    }
}

struct StorableValue;

impl StorableValue {
    fn from(value: Value) -> (String, ExtractedFields) {
        let data = serde_json::to_string(&value).unwrap();
        match ExtractedFields::extract(value) {
            Some(v) => (data, v),
            None => {
                panic!("failed to store de for {data}")
            }
        }
    }
}

fn delocalise(value: Value, language: &'static str) -> Value {
    match value {
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(|value| delocalise(value, language))
                .collect(),
        ),
        Value::Object(obj) => {
            if obj.contains_key("de") || obj.contains_key("en") {
                obj.get(language)
                    .cloned()
                    .unwrap_or(Value::String("".to_string()))
            } else {
                Value::Object(
                    obj.into_iter()
                        .map(|(key, value)| (key, delocalise(value, language)))
                        .filter(|(key, _)| key != "de" && key != "en")
                        .collect(),
                )
            }
        }
        a => a,
    }
}

struct DelocalisedValues {
    key: String,
    de: Value,
    en: Value,
}

impl From<(String, Value)> for DelocalisedValues {
    fn from((key, value): (String, Value)) -> Self {
        Self {
            de: delocalise(value.clone(), "de"),
            en: delocalise(value.clone(), "en"),
            key,
        }
    }
}

impl DelocalisedValues {
    async fn store(self, tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<(), sqlx::Error> {
        let key = self.key.clone(); // has to be here due to livetimes somehow
        let (data, fields) = StorableValue::from(self.de);
        sqlx::query!(
            r#"INSERT INTO de(key,data,name,tumonline_room_nr,type,type_common_name,lat,lon)
            VALUES (?,?,?,?,?,?,?,?)"#,
            key,
            data,
            fields.name,
            fields.tumonline_room_nr,
            fields.r#type,
            fields.type_common_name,
            fields.lat,
            fields.lon,
        )
        .execute(&mut **tx)
        .await?;

        let (data, fields) = StorableValue::from(self.en);
        sqlx::query!(
            r#"INSERT INTO en(key,data,name,tumonline_room_nr,type,type_common_name,lat,lon)
            VALUES (?,?,?,?,?,?,?,?)"#,
            self.key,
            data,
            fields.name,
            fields.tumonline_room_nr,
            fields.r#type,
            fields.type_common_name,
            fields.lat,
            fields.lon,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
pub(crate) async fn load_all_to_db(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let raw_tasks = reqwest::get(format!("{cdn_url}/api_data.json"))
        .await?
        .json::<HashMap<String, Value>>()
        .await?;
    let start = Instant::now();
    let mut index = 0;
    let mut tx = pool.begin().await?;
    for task in raw_tasks.into_iter().map(DelocalisedValues::from) {
        task.store(&mut tx).await?;
        index += 1;
        let elapsed = start.elapsed();
        info!("{index} in {elapsed:?} => avg: {:?}", elapsed / index);
    }
    tx.commit().await?;
    info!("loaded data in {elapsed:?}", elapsed = start.elapsed());

    Ok(())
}
