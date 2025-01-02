use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::Error::RowNotFound;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::error;

use crate::localisation;

use crate::db::location::{
    ComputedProperties, Image, Link, Location, LocationKeyAlias, Operator, OverlayMapEntry,
    ParentLocation, RankingFactor, RoomfinderMapEntry, Source,
};
use crate::db::sections::{BuildingSection, RoomSection};
#[expect(
    unused_imports,
    reason = "has to be imported as otherwise utoipa generates incorrect code"
)]
use serde_json::json;

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
    let resolved_alias = match get_alias_and_redirect(&data.pool, &id).await {
        Ok(Some((i, r, a))) => (i, r, a),
        Ok(None) => {
            return HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Not found")
        }
        Err(e) => {
            error!(error = ?e, id, "error requesting alias");
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Could not get details for this entry, please try again later");
        }
    };
    let response =
        LocationDetailsResponse::fetch_one(&data.pool, resolved_alias, args.should_use_english())
            .await;
    match response {
        Ok(response) => HttpResponse::Ok()
            .insert_header(CacheControl(vec![
                CacheDirective::MaxAge(24 * 60 * 60), // valid for 1d
                CacheDirective::Public,
            ]))
            .json(response),
        Err(e) => {
            error!(error = ?e, id, "Error getting details");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("could not get details for this entry, please try again later")
        }
    }
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    imgs: Option<Vec<ImageInfoResponse>>,
    ranking_factors: RankingFactorsResponse,
    /// Where we got our data from, should be displayed at the bottom of any page containing this data
    sources: SourcesResponse,
    /// The url, this item should be displayed at.
    ///
    /// Present on both redirects and normal entries, to allow for the common /view/:id path
    #[schema(examples("/room/5606.EG.036"))]
    redirect_url: Option<String>,
    /// Coordinate of the location
    coords: CoordinateResponse,
    /// Print or overlay maps for said location
    maps: MapsResponse,
    /// informations for different sectons on the page like the
    /// - buildings overview,
    /// - rooms overview and
    /// - featured view
    #[serde(skip_serializing_if = "Option::is_none")]
    sections: Option<SectionsResponse>,
}
impl LocationDetailsResponse {
    async fn fetch_one(
        pool: &PgPool,
        (id, redirect_url, aliases): (String, String, Vec<String>),
        should_use_english: bool,
    ) -> anyhow::Result<Self> {
        let location = Location::fetch_optional(pool, &id, should_use_english)
            .await?
            .expect("locations with a valid alias have to have a location");
        let props = ComputedProperties::fetch_one(pool, &id).await?;
        // todo: actually use and display the usage
        //let _usage = match location.usage_id {
        //    Some(usage_id) => Usage::fetch_optional(pool, usage_id).await?,
        //    None => None,
        //};
        let operator = match location.operator_id {
            Some(operator_id) => {
                Operator::fetch_optional(pool, operator_id, should_use_english).await?
            }
            None => None,
        };
        let parents = ParentLocation::fetch_all(pool, &id).await?;
        let overlays = OverlayMapEntry::fetch_all(pool, &id).await?;
        let roomfinder = RoomfinderMapEntry::fetch_all(pool, &id).await?;
        let buildings_sections = BuildingSection::fetch_all(pool, &id).await?;
        let rooms_sections = RoomSection::fetch_all(pool, &id).await?;
        let images = Image::fetch_all(pool, &id).await?;
        Ok(LocationDetailsResponse {
            id: id.clone(),
            r#type: LocationTypeResponse::from(location.r#type),
            type_common_name: location.type_common_name,
            name: location.name,
            coords: CoordinateResponse {
                lat: location.lat,
                lon: location.lon,
                source: CoordinateSourceResponse::from(location.coordinate_source),
                accuracy: match location.coordinate_accuracy.unwrap_or_default().as_str() {
                    "building" => Some(CoordinateAccuracyResponse::Building),
                    _ => None,
                },
            },
            aliases,
            parents: parents.clone().into_iter().flat_map(|p| p.id).collect(),
            parent_names: parents.clone().into_iter().flat_map(|p| p.name).collect(),
            props: PropsResponse {
                operator: operator.map(OperatorResponse::from),
                computed: props.clone().into_iter().collect(),
                links: Link::fetch_all(pool, &id, should_use_english)
                    .await?
                    .into_iter()
                    .map(PossibleURLRefResponse::from)
                    .collect(),
                comment: location.comment,
                calendar_url: location.calendar_url,
                building_codes: props.building_codes,
                address: props.address,
                postcode: props.postcode,
                city: props.city,
                level: props.level,
                arch_name: props.arch_name,
                room_cnt: props.room_cnt,
                room_cnt_without_corridors: props.room_cnt_without_corridors,
                building_cnt: props.building_cnt,
            },
            ranking_factors: RankingFactorsResponse::from(
                RankingFactor::fetch_one(pool, &id).await?,
            ),
            sources: SourcesResponse::from(Source::fetch_all(pool, &id).await?),
            redirect_url: Some(redirect_url),
            maps: MapsResponse::from((overlays, roomfinder)),
            imgs: if images.is_empty() {
                Some(images.into_iter().map(ImageInfoResponse::from).collect())
            } else {
                None
            },
            sections: if buildings_sections.is_empty() || rooms_sections.is_empty() {
                Some(SectionsResponse {
                    buildings_overview: if buildings_sections.is_empty() {
                        Some(BuildingsOverviewResponse::from(buildings_sections))
                    } else {
                        None
                    },
                    rooms_overview: if rooms_sections.is_empty() {
                        Some(RoomsOverviewResponse::from(rooms_sections))
                    } else {
                        None
                    },
                    featured_overview: None,
                })
            } else {
                None
            },
        })
    }
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
impl From<String> for LocationTypeResponse {
    fn from(value: String) -> Self {
        match value.as_str() {
            "room" => LocationTypeResponse::Room,
            "building" => LocationTypeResponse::Building,
            "joined_building" => LocationTypeResponse::JoinedBuilding,
            "area" => LocationTypeResponse::Area,
            "site" => LocationTypeResponse::Site,
            "campus" => LocationTypeResponse::Campus,
            "poi" => LocationTypeResponse::Poi,
            _ => LocationTypeResponse::Other,
        }
    }
}

/// Operator of a location
#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
struct OperatorResponse {
    /// ID of the operator
    #[schema(examples(51901))]
    id: i32,
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
impl From<Operator> for OperatorResponse {
    fn from(value: Operator) -> Self {
        OperatorResponse {
            id: value.id.expect("sqlx bug, cannot be none"),
            url: value.url.expect("sqlx bug, cannot be none"),
            code: value.code.expect("sqlx bug, cannot be none"),
            name: value.name.expect("sqlx bug, cannot be none"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct SectionsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    buildings_overview: Option<BuildingsOverviewResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rooms_overview: Option<RoomsOverviewResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
impl From<BuildingSection> for BuildingsOverviewItemResponse {
    fn from(value: BuildingSection) -> Self {
        BuildingsOverviewItemResponse {
            id: value.id.expect("sqlx bug, cannot be null"),
            name: value.name.expect("sqlx bug, cannot be null"),
            subtext: value.subtext.expect("sqlx bug, cannot be null"),
            thumb: value.thumb,
        }
    }
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
    n_visible: usize,
}
impl From<Vec<BuildingSection>> for BuildingsOverviewResponse {
    fn from(value: Vec<BuildingSection>) -> Self {
        let n_visible = value.iter().filter_map(|v| v.visible).count();
        BuildingsOverviewResponse {
            entries: value
                .into_iter()
                .map(BuildingsOverviewItemResponse::from)
                .collect(),
            n_visible,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct RoomsOverviewUsageChildResponse {
    id: String,
    name: String,
}
impl From<RoomSection> for RoomsOverviewUsageChildResponse {
    fn from(value: RoomSection) -> Self {
        RoomsOverviewUsageChildResponse {
            id: value.id,
            name: value.name,
        }
    }
}
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct RoomsOverviewUsageResponse {
    name: String,
    count: usize,
    children: Vec<RoomsOverviewUsageChildResponse>,
}
impl From<(String, Vec<RoomSection>)> for RoomsOverviewUsageResponse {
    fn from((usage, rooms): (String, Vec<RoomSection>)) -> Self {
        RoomsOverviewUsageResponse {
            name: usage,
            count: rooms.len(),
            children: rooms
                .into_iter()
                .map(RoomsOverviewUsageChildResponse::from)
                .collect(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct RoomsOverviewResponse {
    usages: Vec<RoomsOverviewUsageResponse>,
}
impl From<HashMap<String, Vec<RoomSection>>> for RoomsOverviewResponse {
    fn from(value: HashMap<String, Vec<RoomSection>>) -> Self {
        RoomsOverviewResponse {
            usages: value
                .into_iter()
                .map(RoomsOverviewUsageResponse::from)
                .collect(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct FeaturedOverviewResponse {
    entries: Vec<FeaturedOverviewItemResponse>,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct MapsResponse {
    /// type of the Map that should be shown by default
    ///
    /// Only ever `interactive`
    #[schema(deprecated)]
    default: DefaultMapsResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    roomfinder: Option<RoomfinderMapResponse>,
    /// `None` would mean no overlay maps are displayed by default.
    /// For rooms, you should add a warning that no floor map is available for this room
    #[serde(skip_serializing_if = "Option::is_none")]
    overlays: Option<OverlayMapsResponse>,
}
impl From<(Vec<OverlayMapEntry>, Vec<RoomfinderMapEntry>)> for MapsResponse {
    fn from((overlay, roomfinder): (Vec<OverlayMapEntry>, Vec<RoomfinderMapEntry>)) -> Self {
        MapsResponse {
            default: DefaultMapsResponse::Interactive,
            roomfinder: if roomfinder.is_empty() {
                None
            } else {
                Some(RoomfinderMapResponse::from(roomfinder))
            },
            overlays: if overlay.is_empty() {
                None
            } else {
                Some(OverlayMapsResponse::from(overlay))
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct RoomfinderMapResponse {
    /// The id of the map, that should be shown as a default
    #[schema(examples("rf142"))]
    default: String,
    available: Vec<RoomfinderMapEntryResponse>,
}
impl From<Vec<RoomfinderMapEntry>> for RoomfinderMapResponse {
    fn from(value: Vec<RoomfinderMapEntry>) -> Self {
        debug_assert!(value
            .iter()
            .any(|v| v.selected_by_default.unwrap_or_default()));
        RoomfinderMapResponse {
            default: value
                .iter()
                .find(|v| v.selected_by_default.unwrap_or_default())
                .map(|v| v.id.clone().expect("sqlx bug, cannot be none"))
                .expect("asserted to be there beforehand"),
            available: value
                .into_iter()
                .map(RoomfinderMapEntryResponse::from)
                .collect(),
        }
    }
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
impl From<RoomfinderMapEntry> for RoomfinderMapEntryResponse {
    fn from(value: RoomfinderMapEntry) -> Self {
        RoomfinderMapEntryResponse {
            name: value.name.expect("sqlx bug, cannot be none"),
            id: value.id.expect("sqlx bug, cannot be none"),
            scale: value.scale.expect("sqlx bug, cannot be none").to_string(),
            height: value.height.expect("sqlx bug, cannot be none"),
            width: value.width.expect("sqlx bug, cannot be none"),
            x: value.x.expect("sqlx bug, cannot be none"),
            y: value.y.expect("sqlx bug, cannot be none"),
            source: value.source.expect("sqlx bug, cannot be none"),
            file: value.file.expect("sqlx bug, cannot be none"),
        }
    }
}
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct OverlayMapsResponse {
    /// The floor-id of the map, that should be shown as a default.
    ///
    /// `null` means:
    /// - We suggest, you don't show a map by default.
    /// - This is only the case for buildings or other such entities.
    ///   This is not done for rooms, if we know where they are and a map exists.
    #[schema(example = 0)]
    default: Option<i32>,
    available: Vec<OverlayMapEntryResponse>,
}
impl From<Vec<OverlayMapEntry>> for OverlayMapsResponse {
    fn from(value: Vec<OverlayMapEntry>) -> Self {
        OverlayMapsResponse {
            default: value
                .iter()
                .find(|v| v.selected_by_default.unwrap_or_default())
                .map(|v| v.id.expect("sqlx bug, cannot be none")),
            available: value
                .into_iter()
                .map(OverlayMapEntryResponse::from)
                .collect(),
        }
    }
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
impl From<OverlayMapEntry> for OverlayMapEntryResponse {
    fn from(value: OverlayMapEntry) -> Self {
        let lon = value.coordinates_lon.expect("only null because of sqlx");
        let lat = value.coordinates_lat.expect("only null because of sqlx");
        debug_assert!(lon.len() == 4);
        debug_assert!(lat.len() == 4);
        OverlayMapEntryResponse {
            id: value.id.expect("only null because of sqlx"),
            floor: value.floor.expect("only null because of sqlx"),
            name: value.name.expect("only null because of sqlx"),
            file: value.file.expect("only null because of sqlx"),
            coordinates: [
                (lon[0], lat[0]),
                (lon[1], lat[1]),
                (lon[2], lat[2]),
                (lon[3], lat[3]),
            ],
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(deprecated)]
enum DefaultMapsResponse {
    /// interactive maps should be shown first
    #[default]
    Interactive,
    /// roomfinder maps should be shown first
    Roomfinder,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct ExtraComputedPropResponse {
    #[schema(examples("Genauere Angaben"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    header: Option<String>,
    #[schema(examples("for exams: 102 in tight, 71 in wide, 49 in corona"))]
    body: String,
    #[schema(examples("data based on a Survey of chimneysweeps"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
pub struct ComputedPropResponse {
    #[schema(examples("Raumkennung"))]
    name: String,
    #[schema(examples("5602.EG.001"))]
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<ExtraComputedPropResponse>,
}

impl IntoIterator for ComputedProperties {
    type Item = ComputedPropResponse;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        let mut result = vec![];
        if let Some(building_codes) = self.building_codes {
            result.push(ComputedPropResponse {
                name: "Gebäudekennung".to_string(),
                text: building_codes,
                extra: None,
            })
        }
        if let Some(address) = self.address {
            if let Some(postcode) = self.postcode {
                if let Some(city) = self.city {
                    result.push(ComputedPropResponse {
                        name: "Adresse".to_string(),
                        text: format!("{address}, {postcode} {city}"),
                        extra: None,
                    });
                }
            }
        }
        if let Some(level) = self.level {
            result.push(ComputedPropResponse {
                name: "Stockwerk".to_string(),
                text: level,
                extra: None,
            });
        }
        if let Some(arch_name) = self.arch_name {
            result.push(ComputedPropResponse {
                name: "Architekten-Name".to_string(),
                text: arch_name,
                extra: None,
            });
        }
        if let Some(room_cnt) = self.room_cnt {
            if let Some(room_cnt_without_corridors) = self.room_cnt_without_corridors {
                result.push(ComputedPropResponse {
                    name: "Stockwerk".to_string(),
                    text: format!("{room_cnt}, ({room_cnt_without_corridors} ohne Flure etc.)"),
                    extra: None,
                });
            } else {
                result.push(ComputedPropResponse {
                    name: "Stockwerk".to_string(),
                    text: room_cnt.to_string(),
                    extra: None,
                });
            }
        }
        if let Some(building_cnt) = self.building_cnt {
            result.push(ComputedPropResponse {
                name: "Anzahl Gebäude".to_string(),
                text: building_cnt.to_string(),
                extra: None,
            })
        }
        result.into_iter()
    }
}

/// Data for the info-card table
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct PropsResponse {
    /// The operator of the room
    #[serde(skip_serializing_if = "Option::is_none")]
    operator: Option<OperatorResponse>,
    computed: Vec<ComputedPropResponse>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    links: Vec<PossibleURLRefResponse>,
    /// A comment to show to an entry.
    ///
    /// It is used in the rare cases, where some aspect about the room/.. or its translation are misleading.
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    /// Link to the calendar of the room
    #[schema(examples(
        "https://campus.tum.de/tumonline/tvKalender.wSicht?cOrg=19691&cRes=12543&cReadonly=J",
        "https://campus.tum.de/tumonline/tvKalender.wSicht?cOrg=19691&cRes=12559&cReadonly=J"
    ))]
    #[serde(skip_serializing_if = "Option::is_none")]
    calendar_url: Option<String>,
    /// Gebäudekennung
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(examples("16xx"))]
    building_codes: Option<String>,
    /// Adresse
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(examples("Schellingstr. 4"))]
    address: Option<String>,
    /// Postcode
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(examples(80799), minimum = 1, maximum = 99999)]
    postcode: Option<i32>,
    /// City
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(examples("München"))]
    city: Option<String>,
    /// Stockwerk
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(examples("1 (1. OG + 1 Zwischengeschoss)"))]
    level: Option<String>,
    /// Architekten-Name
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(examples("N1101"))]
    arch_name: Option<String>,
    /// Anzahl Räume mit "Fake-Räume" wie Flure etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(examples(147), minimum = 1)]
    room_cnt: Option<i32>,
    /// Anzahl Räume ohne "Fake-Räume" wie Flure etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(examples(105), minimum = 1)]
    room_cnt_without_corridors: Option<i32>,
    /// Anzahl Gebäude
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(examples(31), minimum = 1)]
    building_cnt: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
struct SourceResponse {
    /// Name of the provider
    #[schema(example = "NavigaTUM")]
    name: String,
    /// Url of the provider
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "https://nav.tum.de")]
    url: Option<String>,
}
impl From<Source> for SourceResponse {
    fn from(value: Source) -> Self {
        SourceResponse {
            name: value.name.expect("name should be present"),
            url: value.url,
        }
    }
}
/// Where we got our data from, should be displayed at the bottom of any page containing this data
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct SourcesResponse {
    /// Was this entry patched by us? (e.g. to fix a typo in the name/...)
    /// If so, we should not display the source, as it is not the original source.
    #[serde(skip_serializing_if = "Option::is_none")]
    patched: Option<bool>, // default = false
    /// What is the basis of the data we have
    base: Vec<SourceResponse>,
}
impl From<Vec<Source>> for SourcesResponse {
    fn from(values: Vec<Source>) -> Self {
        let patched = values.iter().any(|v| v.patched == Some(true));
        SourcesResponse {
            patched: if patched { Some(true) } else { None },
            base: values.into_iter().map(SourceResponse::from).collect(),
        }
    }
}

/// The information you need to request Images from the `/cdn/{size}/{id}_{counter}.webp` endpoint
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct ImageInfoResponse {
    /// The name of the image file.
    /// consists of {building_id}_{image_id}.webp, where image_id is a counter starting at 0
    #[schema(examples("mi_0.webp"))]
    name: String,
    author: URLRefResponse,
    source: PossibleURLRefResponse,
    license: PossibleURLRefResponse,
}
impl From<Image> for ImageInfoResponse {
    fn from(value: Image) -> Self {
        ImageInfoResponse {
            name: value.name.expect("sqlx bug, cannot be none"),
            author: URLRefResponse {
                url: value.author_url,
                text: value.author_text.expect("sqlx bug, cannot be none"),
            },
            source: PossibleURLRefResponse {
                url: value.source_url,
                text: value.source_text.expect("sqlx bug, cannot be none"),
            },
            license: PossibleURLRefResponse {
                url: value.license_url,
                text: value.license_text.expect("sqlx bug, cannot be none"),
            },
        }
    }
}

/// A link with a localized link text and url
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct PossibleURLRefResponse {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

impl From<Link> for PossibleURLRefResponse {
    fn from(link: Link) -> Self {
        PossibleURLRefResponse {
            text: link.text.expect("text should be present"),
            url: link.url,
        }
    }
}

/// A link with a localized link text and url
#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct URLRefResponse {
    text: String,
    url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, utoipa::ToSchema)]
struct RankingFactorsResponse {
    #[schema(minimum = 0)]
    rank_combined: i32,
    #[schema(minimum = 0)]
    rank_type: i32,
    #[schema(minimum = 0)]
    rank_usage: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(minimum = 0)]
    rank_boost: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(minimum = 0)]
    rank_custom: Option<i32>,
}
impl From<RankingFactor> for RankingFactorsResponse {
    fn from(ranking_factor: RankingFactor) -> Self {
        RankingFactorsResponse {
            rank_combined: ranking_factor.rank_combined.expect("rank_combined exists"),
            rank_type: ranking_factor.rank_type.expect("rank_type exists"),
            rank_usage: ranking_factor.rank_usage.expect("rank_usage exists"),
            rank_boost: ranking_factor.rank_boost,
            rank_custom: ranking_factor.rank_custom,
        }
    }
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
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
impl From<String> for CoordinateSourceResponse {
    fn from(source: String) -> Self {
        match source.as_str() {
            "navigatum" => Self::Navigatum,
            "roomfinder" => Self::Roomfinder,
            _ => Self::Inferred,
        }
    }
}

#[tracing::instrument(skip(pool))]
async fn get_alias_and_redirect(
    pool: &PgPool,
    query: &str,
) -> anyhow::Result<Option<(String, String, Vec<String>)>> {
    match LocationKeyAlias::fetch_all(pool, query).await {
        Ok(d) => {
            let aliases = d
                .clone()
                .into_iter()
                .map(|a| a.key)
                .collect::<Vec<String>>();
            let redirect_url = match d.len() {
                0 => return Ok(None), // not key or alias
                1 => extract_redirect_exact_match(&d[0].r#type, &d[0].visible_id),
                _ => {
                    format!("/search?q={}", aliases.join("+"))
                }
            };
            let probable_id = d[0].key.clone();
            Ok(Some((probable_id, redirect_url, aliases)))
        }
        Err(RowNotFound) => Ok(None),
        Err(e) => Err(e.into()),
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
