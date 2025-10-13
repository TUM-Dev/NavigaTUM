use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{HttpResponse, get, web};
use serde::{Deserialize, Serialize};
use sqlx::Error::RowNotFound;
use sqlx::PgPool;
use tracing::error;

use crate::localisation::{self, LanguageOptions};

#[expect(
    unused_imports,
    reason = "has to be imported as otherwise utoipa generates incorrect code"
)]
use serde_json::json;

#[derive(Debug, Clone)]
#[allow(dead_code)] // false positive. Clippy can't detect this due to macros
pub struct LocationKeyAlias {
    pub key: String,
    pub visible_id: String,
    pub r#type: String,
}

#[derive(Deserialize, utoipa::IntoParams)]
struct DetailsPathParams {
    /// ID of the location
    id: String,
}

/// Get entry-details
///
/// This returns the full data available for the entry (room/building).
///
/// This is more data, that should be supplied once a user clicks on an entry.
/// Preloading this is not an issue on our end, but keep in mind bandwith constraints on your side.
/// The data can be up to 50kB (using gzip) or 200kB unzipped.
/// More about this data format is described in the NavigaTUM-data documentation
#[utoipa::path(
    tags=["locations"],
    params(DetailsPathParams, localisation::LangQueryArgs),
    responses(
        (status = 200, description = "**Details** about the **location**", body= LocationDetailsResponse, content_type="application/json"),
        (status = 400, description = "**Bad request.** Make sure that requested item ID is not empty and not longer than 255 characters", body = String, content_type = "text/plain", example = "Invalid ID"),
        (status = 404, description = "**Not found.** Make sure that requested item exists", body = String, content_type = "text/plain", example = "Not found"),
    )
)]
#[get("/api/locations/{id}")]
pub async fn get_handler(
    params: web::Path<DetailsPathParams>,
    web::Query(args): web::Query<localisation::LangQueryArgs>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let id = params
        .id
        .replace(|c: char| c.is_whitespace() || c.is_control(), "");
    if params.id.is_empty() || params.id.len() > 255 {
        return HttpResponse::BadRequest()
            .content_type("text/plain")
            .body("Invalid ID");
    }

    let Some((probable_id, redirect_url)) = get_alias_and_redirect(&data.pool, &id).await else {
        return HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Not found");
    };
    let result = if args.lang == LanguageOptions::En {
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
                        error!(error = ?e, id,"cannot serialise detail");
                        HttpResponse::InternalServerError()
                            .content_type("text/plain")
                            .body("Failed to fetch details, please try again later")
                    }
                    Ok(mut res) => {
                        res.redirect_url = redirect_url;
                        HttpResponse::Ok()
                            .insert_header(CacheControl(vec![
                                CacheDirective::MaxAge(24 * 60 * 60), // valid for 1d
                                CacheDirective::Public,
                            ]))
                            .json(res)
                    }
                }
            } else {
                HttpResponse::NotFound()
                    .content_type("text/plain")
                    .body("Not found")
            }
        }
        Err(e) => {
            error!(error = ?e, probable_id, "Error requesting details");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Internal Server Error")
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct LocationDetailsResponse {
    /// The id, that was requested
    #[schema(examples("5606.EG.036"))]
    id: String,
    /// The type of the entry
    r#type: LocationTypeResponse,
    /// The type of the entry in a human-readable form
    #[schema(examples("Büro"))]
    type_common_name: String,
    /// The name of the entry in a human-readable form
    #[schema(examples("5606.EG.036 (Büro Fachschaft Mathe Physik Informatik Chemie / MPIC)"))]
    name: String,
    /// A list of alternative ids for this entry.
    ///
    /// Not to be confused with
    /// - `id` which is the unique identifier or
    /// - `visual-id` which is an alternative identifier for the entry (only displayed in the URL).
    #[schema(examples(json!(["26503@5406"])))]
    aliases: Vec<String>,
    /// The ids of the parents.
    ///
    /// They are ordered as they would appear in a Breadcrumb menu.
    /// See `parent_names` for their human names.
    #[schema(min_items=1, examples(json!(["root","garching","mi", "5602"])))]
    parents: Vec<String>,
    /// The ids of the parents.
    ///
    /// They are ordered as they would appear in a Breadcrumb menu.
    /// See `parents` for their actual ids.
    #[schema(min_items=1, examples(json!(["Standorte","Garching Forschungszentrum","Fakultät Mathematik & Informatik (FMI oder MI)", "Finger 06 (BT06)"])))]
    parent_names: Vec<String>,
    /// Data for the info-card table
    props: PropsResponse,
    /// The information you need to request Images from the `/cdn/{size}/{id}_{counter}.webp` endpoint
    ///
    /// TODO: Sometimes missing, sometimes not.. so weird..
    imgs: Option<Vec<ImageInfoResponse>>,
    ranking_factors: RankingFactorsResponse,
    /// Where we got our data from, should be displayed at the bottom of any page containing this data
    sources: SourcesResponse,
    /// The url, this item should be displayed at.
    ///
    /// Present on both redirects and normal entries, to allow for the common /view/:id path
    #[schema(examples("/room/5606.EG.036"))]
    #[serde(default)]
    redirect_url: String,
    /// Coordinate of the location
    coords: CoordinateResponse,
    /// Print or overlay maps for said location
    maps: MapsResponse,
    /// Information for different sections on the page like the
    /// - buildings overview,
    /// - rooms overview and
    /// - featured view
    #[serde(default)]
    sections: SectionsResponse,
}

#[derive(Serialize, Deserialize, Debug, Default, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum LocationTypeResponse {
    #[default]
    Room,
    Building,
    JoinedBuilding,
    Area,
    Site,
    Campus,
    Poi,
    Other,
}

/// Operator of a location
#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
struct OperatorResponse {
    /// ID of the operator
    #[schema(examples(51901))]
    id: u32,
    ///Link to the operator
    #[schema(examples("https://campus.tum.de/tumonline/webnav.navigate_to?corg=51901"))]
    url: String,
    /// designation code of the operator
    #[schema(examples("TUS7000"))]
    code: String,
    /// The full name of the operator (localized). Null for organisations that
    ///  are no longer active (e.g. id=38698), but where the operator has not been
    /// updated in TUMonline.
    #[schema(examples("TUM School of Social Sciences and Technology"))]
    name: String,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct SectionsResponse {
    buildings_overview: Option<BuildingsOverviewResponse>,
    rooms_overview: Option<RoomsOverviewResponse>,
    featured_overview: Option<FeaturedOverviewResponse>,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct BuildingsOverviewItemResponse {
    /// The id of the entry
    id: String,
    /// Human display name
    name: String,
    /// What should be displayed below this Building
    subtext: String,
    /// The thumbnail for the building
    thumb: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct FeaturedOverviewItemResponse {
    /// The id of the entry
    id: String,
    /// Human display name
    name: String,
    /// What should be displayed below this Building
    subtext: String,
    /// The thumbnail for the building
    image_url: String,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct BuildingsOverviewResponse {
    entries: Vec<BuildingsOverviewItemResponse>,
    n_visible: u32,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct RoomsOverviewUsageChildResponse {
    id: String,
    name: String,
}
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct RoomsOverviewUsageResponse {
    name: String,
    count: u32,
    children: Vec<RoomsOverviewUsageChildResponse>,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct RoomsOverviewResponse {
    usages: Vec<RoomsOverviewUsageResponse>,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct FeaturedOverviewResponse {
    entries: Vec<FeaturedOverviewItemResponse>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct MapsResponse {
    /// type of the Map that should be shown by default
    default: DefaultMapsResponse,
    roomfinder: Option<RoomfinderMapResponse>,
    /// `None` would mean no overlay maps are displayed by default.
    /// For rooms, you should add a warning that no floor map is available for this room
    overlays: Option<OverlayMapsResponse>,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct RoomfinderMapResponse {
    /// The id of the map, that should be shown as a default
    #[schema(examples("rf142"))]
    default: String,
    available: Vec<RoomfinderMapEntryResponse>,
}
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct RoomfinderMapEntryResponse {
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
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct OverlayMapsResponse {
    /// The floor-id of the map, that should be shown as a default.
    /// null means:
    /// - We suggest, you don't show a map by default.
    /// - This is only the case for buildings or other such entities and not for rooms, if we know where they are and a map exists
    #[schema(example = 0)]
    default: Option<i32>,
    available: Vec<OverlayMapEntryResponse>,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct OverlayMapEntryResponse {
    /// Machine-readable floor-id of the map.
    ///
    /// Should start with 0 for the ground level (defined by the main entrance) and increase or decrease.
    /// It is not guaranteed that numbers are consecutive or that `1` corresponds to level `01`, because buildings sometimes have more complicated layouts. They are however always in the correct (physical) order.
    #[schema(example = 0)]
    id: i32,
    /// Floor of the Map.
    ///
    /// Should be used for display to the user in selectors.
    /// Matches the floor part of the TUMonline roomcode.
    #[schema(example = "EG")]
    floor: String,
    /// human-readable name of the map
    #[schema(example = "MI Gebäude (EG)")]
    name: String,
    /// filename of the map
    #[schema(example = "webp/rf95.webp")]
    file: String,
    /// Coordinates are four `[lon, lat]` pairs, for the top left, top right, bottom right, bottom left image corners.
    #[schema(min_items = 4, max_items = 4, example = json!([[11.666739,48.263478],[11.669666,48.263125],[11.669222,48.261585],[11.666331,48.261929]]))]
    coordinates: [(f64, f64); 4],
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum DefaultMapsResponse {
    /// interactive maps should be shown first
    #[default]
    Interactive,
    /// roomfinder maps should be shown first
    Roomfinder,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct ExtraComputedPropResponse {
    #[schema(examples("Genauere Angaben"))]
    header: Option<String>,
    #[schema(examples("for exams: 102 in tight, 71 in wide, 49 in corona"))]
    body: String,
    #[schema(examples("data based on a Survey of chimneysweeps"))]
    footer: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct ComputedPropResponse {
    #[schema(examples("Raumkennung"))]
    name: String,
    #[schema(examples("5602.EG.001"))]
    text: String,
    extra: Option<ExtraComputedPropResponse>,
}

/// Data for the info-card table
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct PropsResponse {
    /// The operator of the room
    operator: Option<OperatorResponse>,
    computed: Vec<ComputedPropResponse>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    links: Vec<PossibleURLRefResponse>,
    /// A comment to show to an entry.
    ///
    /// It is used in the rare cases, where some aspect about the room/.. or its translation are misleading.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    comment: String,
    /// Link to the calendar of the room
    #[schema(examples(
        "https://campus.tum.de/tumonline/tvKalender.wSicht?cOrg=19691&cRes=12543&cReadonly=J",
        "https://campus.tum.de/tumonline/tvKalender.wSicht?cOrg=19691&cRes=12559&cReadonly=J"
    ))]
    calendar_url: Option<String>,
    /// A sorted (lowest floor first) list of floors
    ///
    /// For buildings, this may contain multiple floors while rooms usually only have one floor.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    floors: Vec<FloorResponse>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
struct FloorResponse {
    /// Virtual ID for sorting
    ///
    /// `0` represents the ground floor.
    /// Numbers above/below represent where they are relative to the ground floor
    ///
    /// **WARNING**:
    /// This ID is not guaranteed to be stable.
    /// Not across buildings, nor within a building.
    #[schema(examples(-1, 0, 1, 2, 3))]
    id: i32,
    /// Short name of the floor
    #[schema(examples("-1", "0", "Z1"))]
    #[serde(rename(deserialize = "floor"))]
    short_name: String,
    /// Longer name of the floor
    #[schema(examples(
        "1st basement floor",
        "Ground floor",
        "1st mezzanine, above ground floor"
    ))]
    name: String,
    /// How TUMonline names the floor
    #[schema(examples("U1", "EG", "Z1"))]
    tumonline: String,
    /// Type of floor
    #[schema(examples("basement", "ground", "roof", "mezzanine", "tp"))]
    r#type: FloorType,
}

#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum FloorType {
    /// Top most floor floor, if accessible
    Roof,
    /// Any floor above the ground floor
    Upper,
    /// A floor in a that is half a flight of stairs ABOVE the normal level of the ground floor
    ///
    /// In German: "Zwischenebene" / "Mezzanine"
    #[serde(rename(deserialize = "mezzanine"))]
    SemiUpper,
    /// The normal level of the building
    Ground,
    /// A floor in a that is half a flight of stairs BELOW the normal level of the ground floor
    ///
    /// In German: "Tiefparterre"
    #[serde(rename(deserialize = "tp"))]
    SemiBasement,
    /// Full floors below the ground floor
    Basement,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
struct SourceResponse {
    /// Name of the provider
    #[schema(example = "NavigaTUM")]
    name: String,
    /// Url of the provider
    #[schema(example = "https://nav.tum.de")]
    url: Option<String>,
}
/// Where we got our data from, should be displayed at the bottom of any page containing this data
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct SourcesResponse {
    /// Was this entry patched by us? (e.g. to fix a typo in the name/...)
    /// If so, we should not display the source, as it is not the original source.
    patched: Option<bool>, // default = false
    /// What is the basis of the data we have
    base: Vec<SourceResponse>,
}

/// The information you need to request Images from the `/cdn/{size}/{id}_{counter}.webp` endpoint
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct ImageInfoResponse {
    /// The name of the image file.
    /// consists of {building_id}_{image_id}.webp, where image_id is a counter starting at 0
    #[schema(examples("mi_0.webp"))]
    name: String,
    author: URLRefResponse,
    license: PossibleURLRefResponse,
}

/// A link with a localized link text and url
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct PossibleURLRefResponse {
    text: String,
    url: Option<String>,
}

/// A link with a localized link text and url
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct URLRefResponse {
    text: String,
    url: Option<String>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct RankingFactorsResponse {
    #[schema(minimum = 0)]
    rank_combined: i32,
    #[schema(minimum = 0)]
    rank_type: i32,
    #[schema(minimum = 0)]
    rank_usage: i32,
    #[schema(minimum = 0)]
    rank_boost: Option<i32>,
    #[schema(minimum = 0)]
    rank_custom: Option<i32>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct CoordinateResponse {
    /// Latitude
    #[schema(example = 48.26244490906312)]
    lat: f64,
    /// Longitude
    #[schema(example = 48.26244490906312)]
    lon: f64,
    /// Source of the Coordinates
    #[schema(example = "navigatum")]
    source: CoordinateSourceResponse,
    /// How accurate the coordinate is.
    /// Only present, if it is limited to a degree (e.g. we only know the building)
    #[schema(example = "building")]
    accuracy: Option<CoordinateAccuracyResponse>,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum CoordinateAccuracyResponse {
    #[default]
    Building,
}

#[derive(Serialize, Deserialize, Debug, Default, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum CoordinateSourceResponse {
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
            error!(error = ?e,query,"Error requesting alias");
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
    use tokio::task::LocalSet;
    use tracing::info;

    use super::*;
    use crate::{AppData, setup::tests::PostgresTestContainer};

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
        let app = actix_web::App::new()
            .app_data(web::Data::new(AppData::from(pool)))
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
