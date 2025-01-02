use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[allow(dead_code)] // used for testing out the repo pattern
#[derive(Debug)]
pub struct Location {
    pub last_calendar_scrape_at: Option<DateTime<Utc>>,
    pub lat: f64,
    pub lon: f64,
    pub name: String,
    pub type_common_name: String,
    pub r#type: String,
    pub calendar_url: Option<String>,
    pub tumonline_room_nr: Option<i32>,
    pub coordinate_accuracy: Option<String>,
    pub coordinate_source: String,
    pub comment: Option<String>,
    pub usage_id: Option<i32>,
    pub operator_id: Option<i32>,
}
impl Location {
    #[tracing::instrument(skip(pool))]
    pub async fn fetch_optional(
        pool: &PgPool,
        id: &str,
        should_use_english: bool,
    ) -> sqlx::Result<Option<Self>> {
        if should_use_english {
            sqlx::query_as!(
                Self,
                r#"SELECT last_calendar_scrape_at,lat,lon,name,type_common_name,type,calendar_url,tumonline_room_nr,coordinate_accuracy,coordinate_source,comment,usage_id,operator_id
                FROM en
                WHERE key=$1"#,
                id)
                .fetch_optional(pool).await
        } else {
            sqlx::query_as!(
                Self,
                r#"SELECT last_calendar_scrape_at,lat,lon,name,type_common_name,type,calendar_url,tumonline_room_nr,coordinate_accuracy,coordinate_source,comment,usage_id,operator_id
                FROM de
                WHERE key=$1"#,
                id)
                .fetch_optional(pool).await
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocationKeyAlias {
    pub key: String,
    pub visible_id: String,
    pub r#type: String,
}
impl LocationKeyAlias {
    pub async fn fetch_optional(pool: &PgPool, id: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"
        SELECT key, visible_id, type
        FROM aliases
        WHERE alias = $1 AND key <> alias
        LIMIT 1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }
}
