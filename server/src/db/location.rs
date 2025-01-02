use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[allow(dead_code)] // used for testing out the repo pattern
pub struct RankingFactor {
    pub rank_type: Option<i32>,
    pub rank_combined: Option<i32>,
    pub rank_usage: Option<i32>,
    pub rank_custom: Option<i32>,
    pub rank_boost: Option<i32>,
}
impl RankingFactor {
    #[tracing::instrument(skip(pool))]
    pub async fn fetch_one(pool: &PgPool, id: &str) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"SELECT rank_type, rank_combined, rank_usage, rank_custom, rank_boost
            FROM ranking_factors
            WHERE id=$1"#,
            id
        )
        .fetch_one(pool)
        .await
    }
}

#[allow(dead_code)] // used for testing out the repo pattern
pub struct Operator {
    pub id: Option<i32>,
    pub url: Option<String>,
    pub code: Option<String>,
    pub name: Option<String>,
}
impl Operator {
    #[tracing::instrument(skip(pool))]
    pub async fn fetch_optional(
        pool: &PgPool,
        id: i32,
        should_use_english: bool,
    ) -> sqlx::Result<Option<Self>> {
        if should_use_english {
            sqlx::query_as!(
                Self,
                r#"SELECT id,url,code,name
                FROM operators_en
                WHERE id=$1"#,
                id
            )
            .fetch_optional(pool)
            .await
        } else {
            sqlx::query_as!(
                Self,
                r#"SELECT id,url,code,name
                FROM operators_de
                WHERE id=$1"#,
                id
            )
            .fetch_optional(pool)
            .await
        }
    }
}

#[allow(dead_code)] // used for testing out the repo pattern
pub struct Usage {
    pub name: Option<String>,
    pub din_277: Option<String>,
    pub din_277_desc: Option<String>,
}
impl Usage {
    #[tracing::instrument(skip(pool))]
    pub async fn fetch_optional(pool: &PgPool, id: i32) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT name,din_277,din_277_desc
            FROM usages
            WHERE usage_id=$1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }
}

pub struct RoomfinderMapEntry {
    pub name: Option<String>,
    pub id: Option<String>,
    pub scale: Option<i32>,
    pub height: Option<i32>,
    pub width: Option<i32>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub source: Option<String>,
    pub file: Option<String>,
    pub selected_by_default: Option<bool>,
}
impl RoomfinderMapEntry {
    #[tracing::instrument(skip(pool))]
    pub async fn fetch_all(pool: &PgPool, id: &str) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT name,id,scale,height,width,x,y,source,file,selected_by_default
            FROM roomfinder_maps
            WHERE key=$1",
            id
        )
        .fetch_all(pool)
        .await
    }
}
pub struct OverlayMapEntry {
    pub id: Option<i32>,
    pub floor: Option<String>,
    pub name: Option<String>,
    pub file: Option<String>,
    pub coordinates_lon: Option<Vec<f64>>,
    pub coordinates_lat: Option<Vec<f64>>,
    pub selected_by_default: Option<bool>,
}
impl OverlayMapEntry {
    #[tracing::instrument(skip(pool))]
    pub async fn fetch_all(pool: &PgPool, id: &str) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id,floor,name,file,coordinates_lon,coordinates_lat,selected_by_default
            FROM overlay_maps
            WHERE key=$1",
            id
        )
        .fetch_all(pool)
        .await
    }
}

#[allow(dead_code)] // used for testing out the repo pattern
#[derive(Debug, Clone)]
pub struct ComputedProperties {
    /// Gebäudekennung
    ///
    /// Example: `16xx`
    pub building_codes: Option<String>,
    /// Adresse
    ///
    /// Example: `Schellingstr. 4`
    pub address: Option<String>,
    /// Postcode
    ///
    /// Example: `80799`
    pub postcode: Option<i32>,
    /// City
    ///
    /// Example: `München`
    pub city: Option<String>,
    /// Stockwerk
    ///
    /// Example: `1 (1. OG + 1 Zwischengeschoss)`
    pub level: Option<String>,
    /// Architekten-Name
    ///
    /// Example: `N1101`
    pub arch_name: Option<String>,
    /// Anzahl Räume mit "Fake-Räume" wie Flure etc.
    ///
    /// Example: `147`
    pub room_cnt: Option<i32>,
    /// Anzahl Räume ohne "Fake-Räume" wie Flure etc.
    ///
    /// Example: `105`
    pub room_cnt_without_corridors: Option<i32>,
    /// Anzahl Gebäude
    ///
    /// Example: `31`
    pub building_cnt: Option<i32>,
}
impl ComputedProperties {
    #[tracing::instrument(skip(pool))]
    pub async fn fetch_one(pool: &PgPool, id: &str) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"SELECT building_codes,address,city,postcode,level,arch_name,room_cnt,room_cnt_without_corridors,building_cnt
            FROM computed_properties
            WHERE key=$1"#,
            id
        )
        .fetch_one(pool)
        .await
    }
}

#[allow(dead_code)] // used for testing out the repo pattern
pub struct Link {
    pub url: Option<String>,
    pub text: Option<String>,
}
impl Link {
    #[tracing::instrument(skip(pool))]
    pub async fn fetch_all(
        pool: &PgPool,
        id: &str,
        should_use_english: bool,
    ) -> sqlx::Result<Vec<Self>> {
        if should_use_english {
            sqlx::query_as!(
                Self,
                r#"SELECT url,text
                FROM urls_de
                WHERE key=$1"#,
                id
            )
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as!(
                Self,
                r#"SELECT url, text
                FROM urls_de
                WHERE key=$1"#,
                id
            )
            .fetch_all(pool)
            .await
        }
    }
}
#[allow(dead_code)] // used for testing out the repo pattern
pub struct Source {
    pub url: Option<String>,
    pub name: Option<String>,
    pub patched: Option<bool>,
}
impl Source {
    #[tracing::instrument(skip(pool))]
    pub async fn fetch_all(pool: &PgPool, id: &str) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT url, name, patched
            FROM sources
            WHERE key=$1"#,
            id
        )
        .fetch_all(pool)
        .await
    }
}

#[allow(dead_code)] // used for testing out the repo pattern
#[derive(Debug, Clone)]
pub struct ParentLocation {
    pub id: Option<String>,
    pub name: Option<String>,
}
impl ParentLocation {
    #[tracing::instrument(skip(pool))]
    pub async fn fetch_all(pool: &PgPool, id: &str) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT id,name
                FROM parents
                WHERE key=$1"#,
            id
        )
        .fetch_all(pool)
        .await
    }
}

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

#[allow(dead_code)] // used for testing out the repo pattern
#[derive(Debug, Clone)]
pub struct LocationKeyAlias {
    pub key: String,
    pub visible_id: String,
    pub r#type: String,
}
impl LocationKeyAlias {
    #[tracing::instrument(skip(pool))]
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
    pub async fn fetch_all(pool: &PgPool, id: &str) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"
        SELECT key, visible_id, type
        FROM aliases
        WHERE alias = $1 AND key <> alias
        LIMIT 1"#,
            id
        )
        .fetch_all(pool)
        .await
    }
}
#[allow(dead_code)] // used for testing out the repo pattern
pub struct Image {
    pub name: Option<String>,
    pub author_url: Option<String>,
    pub author_text: Option<String>,
    pub source_url: Option<String>,
    pub source_text: Option<String>,
    pub license_url: Option<String>,
    pub license_text: Option<String>,
}
impl Image {
    pub async fn fetch_all(pool: &PgPool, id: &str) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT name,author_url,author_text,source_url,source_text,license_url,license_text
            FROM location_images
            WHERE key = $1"#,
            id
        )
        .fetch_all(pool)
        .await
    }
}
