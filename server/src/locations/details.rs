use actix_web::http::header::{CacheControl, CacheDirective};
use crate::localisation;
use crate::models::LocationKeyAlias;
use actix_web::{get, web, HttpResponse};
use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Error::RowNotFound;
use sqlx::PgPool;
use tracing::error;

#[derive(Serialize, Debug, Clone)]
struct Usage {
    id: i64,
    name: String,
    din_277: String,
    din_277_desc: String,
}

#[derive(Serialize, Debug, Clone)]
struct Operator {
    id: String,
    url: String,
    code: String,
    name: String,
}

#[derive(Serialize, Debug, Clone)]
struct Url {
    name: String,
    url: String,
}

#[derive(Debug, Clone)]
struct DBLocationDetails {
    key: String,
    name: String,
    last_calendar_scrape_at: Option<DateTime<Utc>>,
    calendar_url: Option<String>,
    r#type: String,
    type_common_name: String,
    lat: f64,
    lon: f64,
    coordinate_source: String,
    coordinate_accuracy: String,
    comment: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum LocationType {
    #[default]
    Room,
    Building,
    JoinedBuilding,
    Area,
    Site,
    Campus,
    Poi,
}

#[derive(Serialize, Default)]
struct GetLocationDetails {
    /// The id, that was requested
    id: String,
    /// The type of the entry
    r#type: LocationType,
    /// The type of the entry in a human-readable form
    type_common_name: String,
    /// The name of the entry in a human-readable form
    name: String,
    /// A list of alternative ids for this entry.
    ///
    /// Not to be confused with
    /// - [`id`] which is the unique identifier or
    /// - [`visual-id`] which is an alternative identifier for the entry (only displayed in the URL).
    aliases: Vec<String>,
    /// The ids of the parents.
    /// They are ordered as they would appear in a Breadcrumb menu.
    /// See [`parent_names`] for their human names.
    parents: Vec<String>,
    /// The ids of the parents. They are ordered as they would appear in a Breadcrumb menu.
    /// See [`parents`] for their actual ids.
    parent_names: Vec<String>,
    /// Data for the info-card table
    props: LocationProps,
    /// The information you need to request Images from the /cdn/{size}/{id}_{counter}.webp endpoint
    imgs: Vec<LocationImage>,
    ranking_factors: RankingFactors,
    /// Where we got our data from, should be displayed at the bottom of any page containing this data
    sources: Sources,
    /// The url, this item should be displayed at. Present on both redirects and normal entries, to allow for the common /view/:id path
    redirect_url: String,
    coords: Coordinate,
    maps: Maps,
    sections: Sections,
}

#[derive(Serialize, Default)]
struct Sections {}

#[derive(Serialize, Default)]
struct Maps {}

#[derive(Serialize, Default)]
struct LocationProps {}

#[derive(Serialize, Default)]
struct Sources {}

#[derive(Serialize)]
struct LocationImage {}

#[derive(Serialize, Default)]
struct RankingFactors {
    rank_combined: u32,
    rank_type: u32,
    rank_usage: u32,
    rank_boost: Option<u32>,
    rank_custom: Option<u32>,
}

#[derive(Serialize, Default)]
struct Coordinate {
    lat: f64,
    lon: f64,
    source: CoordinateSource,
    accuracy: Option<CoordinateAccuracy>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CoordinateAccuracy {
    Buiding,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum CoordinateSource {
    Roomfinder,
    #[default]
    Navigatum,
    Inferred,
}

impl TryFrom<DBLocationDetails> for GetLocationDetails {
    type Error = anyhow::Error;

    fn try_from(base: DBLocationDetails) -> anyhow::Result<Self> {
        Ok(Self {
            id: base.key,
            name: base.name,
            r#type: LocationType::des(base.r#type)?,
            type_common_name: base.type_common_name,
            coords: Coordinate {
                lat: base.lat,
                lon: base.lon,
                source: serde_json::from_str(&base.coordinate_source)?,
                accuracy: match base.coordinate_accuracy {
                    Some(a) => CoordinateAccuracy::try_from(a)?,
                    None => None,
                },
            },
            props: LocationProps {
                comment: base.comment,
                last_calendar_scrape_at: base.last_calendar_scrape_at,
                calendar_url: base.calendar_url,
            },
            ranking_factors: Default::default(),
        })
    }
}

#[get("/{id}")]
pub async fn get_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<localisation::LangQueryArgs>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let id = params
        .into_inner()
        .replace(|c: char| c.is_whitespace() || c.is_control(), "");
    let Some((probable_id, redirect_url)) = get_alias_and_redirect(&data.pool, &id).await else {
        return HttpResponse::NotFound().body("Not found");
    };
    let result = if args.should_use_english() {
        sqlx::query_as!(DBLocationDetails,
            r#"SELECT key,name,last_calendar_scrape_at,calendar_url,type,type_common_name,lat,lon,coordinate_source,rank_type,rank_combined,rank_usage,comment
            FROM en
            WHERE key = $1"#r,
            probable_id)
            .fetch_optional(&data.pool)
            .await
    } else {
        sqlx::query_as!(DBLocationDetails,
            r#"SELECT key,name,last_calendar_scrape_at,calendar_url,type,type_common_name,lat,lon,coordinate_source,rank_type,rank_combined,rank_usage,comment
            FROM de
            WHERE key = $1"#r, probable_id)
            .fetch_optional(&data.pool)
            .await
    };
    match result {
        Ok(d) => match d {
            None => HttpResponse::NotFound().body("Not found"),
            Some(d) => {
                let mut res = GetLocationDetails::from(d);
                res.redirect_url = redirect_url;

                HttpResponse::Ok()
                    .insert_header(CacheControl(vec![
                        CacheDirective::MaxAge(24 * 60 * 60), // valid for 1d
                        CacheDirective::Public,
                    ]))
                    .json(res)
            }
        },
        Err(e) => {
            error!("Error requesting details for {probable_id}: {e:?}");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Internal Server Error")
        }
    }
}

#[tracing::instrument(skip(pool))]
async fn get_alias_and_redirect(pool: &PgPool, query: &str) -> Option<(String, String)> {
    let result = sqlx::query_as!(
        LocationKeyAlias,
        r#"
        SELECT DISTINCT key, visible_id, type
        FROM aliases
        WHERE alias = $1 OR key = $1 "#,
        query
    )
    .fetch_all(pool)
    .await;
    match result {
        Ok(d) => {
            let redirect_url = match d.len() {
                0 => return None, // not key or alias
                1 => extract_redirect_exact_match(&d[0].r#type, &d[0].visible_id),
                _ => {
                    let keys = d
                        .clone()
                        .into_iter()
                        .map(|a| a.key)
                        .collect::<Vec<String>>();
                    format!("/search?q={}", keys.join("+"))
                }
            };
            Some((d[0].key.clone(), redirect_url))
        }
        Err(RowNotFound) => None,
        Err(e) => {
            error!("Error requesting alias for {query}: {e:?}");
            None
        }
    }
}

fn extract_redirect_exact_match(type_: &str, key: &str) -> String {
    match type_ {
        "campus" => format!("/campus/{key}"),
        "site" | "area" => format!("/site/{key}"),
        "building" | "joined_building" => format!("/building/{key}"),
        "room" | "virtual_room" => format!("/room/{key}"),
        "poi" => format!("/poi/{key}"),
        _ => format!("/view/{key}"), // can be triggered if we add a type but don't add it here
    }
}
