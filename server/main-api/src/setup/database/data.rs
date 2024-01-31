use std::collections::HashMap;
use std::time::Instant;

use log::info;
use serde_json::Value;

struct ExtractedFields {
    name: String,
    tumonline_room_nr: Option<i32>,
    r#type: String,
    type_common_name: String,
    lat: f64,
    lon: f64,
}

impl From<HashMap<String, Value>> for ExtractedFields {
    fn from(obj: HashMap<String, Value>) -> Self {
        let props = obj.get("props").unwrap().as_object().unwrap();
        let tumonline_room_nr = props
            .get("tumonline_room_nr")
            .map(|v| v.as_i64().unwrap() as i32);
        let (lat, lon) = match obj.get("coords") {
            Some(coords) => {
                let lat = coords.get("lat").unwrap().as_f64();
                let lon = coords.get("lon").unwrap().as_f64();
                (lat, lon)
            }
            None => (None, None),
        };
        ExtractedFields {
            name: obj.get("name").unwrap().as_str().unwrap().to_string(),
            tumonline_room_nr,
            r#type: obj.get("type").unwrap().as_str().unwrap().to_string(),
            type_common_name: obj
                .get("type_common_name")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            lat: lat.unwrap_or(48.14903),
            lon: lon.unwrap_or(11.56735),
        }
    }
}

struct StorableValue;

impl StorableValue {
    fn from(value: HashMap<String, Value>) -> (String, ExtractedFields) {
        let data = serde_json::to_string(&value).unwrap();
        (data, ExtractedFields::from(value))
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
                    .unwrap_or(Value::String(String::new()))
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
    de: HashMap<String, Value>,
    en: HashMap<String, Value>,
}

impl From<HashMap<String, Value>> for DelocalisedValues {
    fn from(value: HashMap<String, Value>) -> Self {
        Self {
            de: value
                .clone()
                .into_iter()
                .map(|(k, v)| (k, delocalise(v.clone(), "de")))
                .collect(),
            en: value
                .clone()
                .into_iter()
                .map(|(k, v)| (k, delocalise(v.clone(), "en")))
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
    async fn store(
        self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::Error> {
        let key = self.key.clone(); // has to be here due to livetimes somehow
        let (data, fields) = StorableValue::from(self.de);
        sqlx::query!(
            r#"INSERT INTO de(key,data,name,tumonline_room_nr,type,type_common_name,lat,lon)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)"#,
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
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)"#,
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
