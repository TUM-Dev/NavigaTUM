use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub key: String,
    pub name: String,
    pub last_calendar_scrape_at: Option<DateTime<Utc>>,
    pub calendar_url: Option<String>,
    pub r#type: String,
    pub type_common_name: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // false positive. Clippy can't detect this due to macros
pub struct LocationKeyAlias {
    pub key: String,
    pub visible_id: String,
    pub r#type: String,
}
