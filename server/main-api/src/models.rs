use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Location {
    pub key: String,
    pub name: String,
    pub last_calendar_scrape_at: Option<DateTime<Utc>>,
    pub tumonline_room_nr: Option<i32>,
    pub r#type: String,
    pub type_common_name: String,
    pub lat: f64,
    pub lon: f64,
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct LocationKeyAlias {
    pub key: String,
    pub visible_id: String,
    pub r#type: String,
}
