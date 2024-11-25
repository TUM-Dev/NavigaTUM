use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::Error::RowNotFound;
use sqlx::PgPool;
use tracing::error;

use crate::localisation;
use crate::models::LocationKeyAlias;

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
        sqlx::query_scalar!("SELECT data FROM en WHERE key = $1", probable_id)
            .fetch_optional(&data.pool)
            .await
    } else {
        sqlx::query_scalar!("SELECT data FROM de WHERE key = $1", probable_id)
            .fetch_optional(&data.pool)
            .await
    };
    match result {
        Ok(d) => {
            if let Some(d) = d {
                let res = serde_json::from_value::<LocationDetailsResponse>(d);
                match res {
                    Err(e) => {
                        error!("cannot serialise {id} because {e:?}");
                        HttpResponse::InternalServerError()
                            .content_type("text/plain")
                            .body("Internal Server Error")
                    }
                    Ok(mut res) => {
                        res.redirect_url = Some(redirect_url);
                        HttpResponse::Ok()
                            .insert_header(CacheControl(vec![
                                CacheDirective::MaxAge(24 * 60 * 60), // valid for 1d
                                CacheDirective::Public,
                            ]))
                            .json(res)
                    }
                }
            } else {
                HttpResponse::NotFound().body("Not found")
            }
        }
        Err(e) => {
            error!("Error requesting details for {probable_id}: {e:?}");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Internal Server Error")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Operator {
    id: u32,
    url: String,
    code: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
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

#[derive(Deserialize, Serialize, Debug, Default)]
struct LocationDetailsResponse {
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
    props: Props,
    /// The information you need to request Images from the /cdn/{size}/{id}_{counter}.webp endpoint
    /// TODO: Sometimes missing, sometimes not.. so weird..
    #[serde(skip_serializing_if = "Option::is_none")]
    imgs: Option<Vec<ImageInfo>>,
    ranking_factors: RankingFactors,
    /// Where we got our data from, should be displayed at the bottom of any page containing this data
    sources: Sources,
    /// The url, this item should be displayed at. Present on both redirects and normal entries, to allow for the common /view/:id path
    redirect_url: Option<String>,
    coords: Coordinate,
    maps: Maps,
    #[serde(skip_serializing_if = "Option::is_none")]
    sections: Option<Sections>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct Sections {
    #[serde(skip_serializing_if = "Option::is_none")]
    buildings_overview: Option<BuildingsOverview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rooms_overview: Option<RoomsOverview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    featured_overview: Option<FeaturedOverview>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct BuildingsOverviewItem {
    /// The id of the entry
    id: String,
    /// Human display name
    name: String,
    /// What should be displayed below this Building
    subtext: String,
    /// The thumbnail for the building
    thumb: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct FeaturedOverviewItem {
    /// The id of the entry
    id: String,
    /// Human display name
    name: String,
    /// What should be displayed below this Building
    subtext: String,
    /// The thumbnail for the building
    image_url: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct BuildingsOverview {
    entries: Vec<BuildingsOverviewItem>,
    n_visible: u32,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct RoomsOverviewUsageChild {
    id: String,
    name: String,
}
#[derive(Deserialize, Serialize, Debug, Default)]
struct RoomsOverviewUsage {
    name: String,
    count: u32,
    children: Vec<RoomsOverviewUsageChild>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct RoomsOverview {
    usages: Vec<RoomsOverviewUsage>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct FeaturedOverview {
    entries: Vec<FeaturedOverviewItem>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct Maps {
    default: DefaultMaps,
    #[serde(skip_serializing_if = "Option::is_none")]
    roomfinder: Option<RoomfinderMap>,
    /// [`None`] would mean no overlay maps are displayed by default.
    /// For rooms, you should add a warning that no floor map is available for this room
    #[serde(skip_serializing_if = "Option::is_none")]
    overlays: Option<OverlayMaps>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct RoomfinderMap {
    /// The id of the map, that should be shown as a default
    /// Example: `rf142`
    default: String,
    available: Vec<RoomfinderMapEntry>,
}
#[derive(Deserialize, Serialize, Debug, Default)]
struct RoomfinderMapEntry {
    /// human-readable name of the map
    name: String,
    /// machine-readable name of the map
    id: String,
    /// Scale of the map. 2000 means 1:2000
    scale: String,
    /// Map image y dimensions
    height: i32,
    /// Map image y dimensions
    width: i32,
    /// x Position on map image
    x: i32,
    /// y Position on map image
    y: i32,
    /// Where the map was imported from
    source: String,
    /// Where the map is stored
    file: String,
}
#[derive(Deserialize, Serialize, Debug, Default)]
struct OverlayMaps {
    /// The floor-id of the map, that should be shown as a default.  
    /// null means:
    /// - We suggest, you don't show a map by default.
    /// - This is only the case for buildings or other such entities and not for rooms, if we know where they are and a map exists
    default: Option<i32>,
    available: Vec<OverlayMapEntry>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct OverlayMapEntry {
    /// machine-readable floor-id of the map.
    /// Should start with 0 for the ground level (defined by the main entrance) and increase or decrease.
    /// It is not guaranteed that numbers are consecutive or that `1` corresponds to level `01`, because buildings sometimes have more complicated layouts. They are however always in the correct (physical) order.
    id: i32,
    /// Floor of the Map.
    /// Should be used for display to the user in selectors.
    /// Matches the floor part of the TUMonline roomcode.
    floor: String,
    /// human-readable name of the map
    name: String,
    /// filename of the map
    file: String,
    /// Coordinates are four `[lon, lat]` pairs, for the top left, top right, bottom right, bottom left image corners.
    coordinates: [(f64, f64); 4],
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
enum DefaultMaps {
    #[default]
    Interactive,
    Roomfinder,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct ExtraComputedProp {
    /// example: `Genauere Angaben`
    #[serde(skip_serializing_if = "Option::is_none")]
    header: Option<String>,
    /// example: `for exams: 102 in tight, 71 in wide, 49 in corona`
    body: String,
    /// example: `data based on a Survey of chimneysweeps`
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct ComputedProp {
    /// example: `Raumkennung`
    name: String,
    /// example: `5602.EG.001`
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<ExtraComputedProp>,
}
#[derive(Deserialize, Serialize, Debug, Default)]
struct Props {
    /// The operator of the room
    #[serde(skip_serializing_if = "Option::is_none")]
    operator: Option<Operator>,
    computed: Vec<ComputedProp>,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    links: Vec<PossibleURLRef>,
    /// A comment to show to an entry.
    /// It is used in the rare cases, where some aspect about the room/.. or its translation are misleading.
    /// An example of a room with a comment is `MW1801`.
    #[serde(skip_serializing_if = "String::is_empty", default = "String::new")]
    comment: String,
    /// link to the calendar of the room
    /// examples:
    /// - 'https://campus.tum.de/tumonline/tvKalender.wSicht?cOrg=19691&cRes=12543&cReadonly=J'
    /// - 'https://campus.tum.de/tumonline/tvKalender.wSicht?cOrg=19691&cRes=12559&cReadonly=J'
    #[serde(skip_serializing_if = "Option::is_none")]
    calendar_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Source {
    /// name of the provider
    name: String,
    /// url of the provider
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}
#[derive(Deserialize, Serialize, Debug, Default)]
struct Sources {
    /// Was this entry patched by us? (e.g. to fix a typo in the name/...)
    /// If so, we should not display the source, as it is not the original source.
    #[serde(skip_serializing_if = "Option::is_none")]
    patched: Option<bool>, // default = false
    /// What is the basis of the data we have
    base: Vec<Source>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct ImageInfo {
    /// The name of the image file. consists of {building_id}_{image_id}.webp, where image_id is a counter starting at 0
    name: String,
    author: URLRef,
    source: PossibleURLRef,
    license: PossibleURLRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<ImageMetadata>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct PossibleURLRef {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct URLRef {
    text: String,
    url: Option<String>,
}

/// Additional data about the images.
/// Does not have to be displayed.
/// All fields are optional.
#[derive(Deserialize, Serialize, Debug, Default)]
struct ImageMetadata {
    ///optional date description
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    ///optional location description
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
    ///optional coordinates in lat,lon
    #[serde(skip_serializing_if = "Option::is_none")]
    geo: Option<String>,
    /// optional in contrast to source this points to the image itself.
    /// You should not use this to request the images, as they are not scaled.
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<String>,
    /// optional caption
    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,
    /// optional headline
    #[serde(skip_serializing_if = "Option::is_none")]
    headline: Option<String>,
    ///  optional the event this image was taken at
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<String>,
    /// optional the event this image is about
    #[serde(skip_serializing_if = "Option::is_none")]
    faculty: Option<String>,
    ///optional the building this image is about
    #[serde(skip_serializing_if = "Option::is_none")]
    building: Option<String>,
    ///  optional the department this image is about
    #[serde(skip_serializing_if = "Option::is_none")]
    department: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct RankingFactors {
    rank_combined: u32,
    rank_type: u32,
    rank_usage: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    rank_boost: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rank_custom: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct Coordinate {
    lat: f64,
    lon: f64,
    source: CoordinateSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    accuracy: Option<CoordinateAccuracy>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
enum CoordinateAccuracy {
    #[default]
    Building,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
enum CoordinateSource {
    #[default]
    Navigatum,
    Roomfinder,
    Inferred,
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::task::LocalSet;
    use tracing::info;

    use super::*;
    use crate::{setup::tests::PostgresTestContainer, AppData};

    /// Allows testing if a modification has changed the output of the details API
    ///
    /// The testcase can be executed via running the following command on main
    /// ```bash
    /// INSTA_OUTPUT=none INSTA_UPDATE=always DATABASE_URL=postgres://postgres:CHANGE_ME@localhost:5432 cargo test --package navigatum-server test_get_handler_unchanged -- --nocapture --include-ignored
    /// ```
    ///
    /// And then running this command on the change
    /// ```bash
    /// DATABASE_URL=postgres://postgres:CHANGE_ME@localhost:5432 cargo insta test --review --package navigatum-server -- test_get_handler_unchanged --nocapture --include-ignored
    /// ```
    ///
    /// This is a *bit* slow, due to using a [`tokio::task::LocalSet`].
    /// Using multiple cores for this might be possible, but optimising this testcase from 10m is currently not worth it
    #[ignore]
    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_get_handler_unchanged() {
        let pg = PostgresTestContainer::new().await;
        pg.load_data_retrying().await;

        let keys: Vec<String> = sqlx::query_scalar("SELECT key FROM de")
            .fetch_all(&pg.pool)
            .await
            .unwrap();
        let all_keys_len = keys.len();
        let mut resolved_cnt = 0_usize;

        for key_chunk in keys.chunks(1000) {
            let tasks = LocalSet::new();
            for key in key_chunk {
                let inner_key = key.clone();
                let inner_pool = pg.pool.clone();
                tasks.spawn_local(async move {
                    check_snapshot(inner_key, inner_pool).await;
                });
            }
            tasks.await;
            resolved_cnt += key_chunk.len();
            info!(
                "processed {resolved_cnt}/{all_keys_len} <=> {percentage}%",
                percentage = 100_f32 * (resolved_cnt as f32) / (all_keys_len as f32)
            );
        }
    }

    async fn check_snapshot(key: String, pool: PgPool) {
        let data = AppData {
            pool,
            meilisearch_initialised: Arc::new(Default::default()),
        };
        let app = actix_web::App::new()
            .app_data(web::Data::new(data))
            .service(get_handler);
        let app = actix_web::test::init_service(app).await;
        let req = actix_web::test::TestRequest::get()
            .uri(&format!("/{key}"))
            .to_request();
        let (_, resp) = actix_web::test::call_service(&app, req).await.into_parts();

        assert_eq!(resp.status().as_u16(), 200);

        let body_box = resp.into_body();
        let body_bytes = actix_web::body::to_bytes(body_box).await.unwrap();
        let body_str = String::from_utf8(body_bytes.into_iter().collect()).unwrap();
        let body_value: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        let mut settings = insta::Settings::clone_current();
        settings.set_sort_maps(true);
        settings.set_snapshot_path("location_get_handler");
        settings.bind(|| {
            insta::assert_json_snapshot!(key.clone(), body_value, {".hash" => 0});
        });
    }
}
