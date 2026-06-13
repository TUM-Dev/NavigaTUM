use chrono::{DateTime, Utc};
use meilisearch_sdk::client::Client;
use parser::TextToken;
use serde::Serialize;
use std::fmt::{self, Debug, Formatter};
use strum::EnumCount as _;
use tracing::error;

use crate::external::meilisearch::{GeoEntryQuery, LocationEntryType, MSHit, UpcomingEvent};
use crate::external::nominatim::Nominatim;
use crate::limited::vec::LimitedVec;
use crate::routes::search::{FormattingConfig, Limits};
use crate::search_executor::parser::ParsedQuery;

mod formatter;
mod highlight;
mod lexer;
mod merger;
mod parser;

/// The facet a [`ResultsSection`] groups - its identity in the merge ordering
/// and the discriminator serialized as the section's `facet` tag. Internal; the
/// wire form is generated from the [`ResultsSection`] variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::EnumCount)]
pub enum ResultFacet {
    Sites,
    Buildings,
    Rooms,
    Pois,
    Lectures,
    Events,
    Addresses,
}

/// A `NavigaTUM` entity search result: a site, building, room, or POI.
///
/// Carries the room-id formatting fields (`parsed_id`/`subtext_bold`) that only
/// make sense for locations; lectures use [`LectureEntry`] and Nominatim
/// addresses use [`AddressEntry`] instead.
#[derive(Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct LocationEntry {
    /// The originating meilisearch hit, kept around so the room formatter can
    /// enrich `parsed_id`/`subtext` after merging. Never serialized. Boxed to
    /// keep this struct from dwarfing the lecture one (the hit is bulky and
    /// out-of-band).
    #[serde(skip)]
    hit: Box<MSHit>,
    /// The id of the location
    #[schema(example = "5510.03.002")]
    id: String,
    /// The type of the entity, resolving to its canonical `/{type}/{id}` route.
    r#type: LocationEntryType,
    /// The display name of the result. Supports highlighting.
    #[schema(example = "5510.03.002 (\x19MW\x17 2001, Empore)")]
    name: String,
    /// Subtext to show below the search result.
    ///
    /// Usually contains the context of where this rooms is located in.
    /// Currently not highlighted.
    #[schema(example = "Maschinenwesen (MW)")]
    subtext: String,
    /// Subtext to show below the search (by default in bold and after the non-bold subtext).
    ///
    /// Usually contains the arch-id of the room, which is another common room id format, and supports highlighting.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "3002@5510")]
    subtext_bold: Option<String>,
    /// This is an optional feature, that is only supported for some rooms.
    ///
    /// It might be displayed instead or before the name, to show that a different room id format has matched, that was probably used.
    /// See the image below for an example.
    /// It will be cropped to a maximum length to not take too much space in UIs.
    /// Supports highlighting.
    #[serde(skip_serializing_if = "Option::is_none")]
    parsed_id: Option<String>,
}

/// A lecture search result, carrying its bilingual titles and upcoming occurrences.
#[derive(Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct LectureEntry {
    /// The id of the lecture
    #[schema(example = "lecture_5f2c…")]
    id: String,
    /// The display name of the result. Supports highlighting.
    #[schema(example = "Einführung in die Informatik 1")]
    name: String,
    /// Subtext to show below the search result.
    ///
    /// Carries the human `stp_type` label (e.g. "Vorlesung").
    #[schema(example = "Vorlesung")]
    subtext: String,
    /// The German title of the lecture.
    #[schema(example = "Einführung in die Informatik 1")]
    title_de: String,
    /// The English title of the lecture.
    #[schema(example = "Introduction to Informatics 1")]
    title_en: String,
    /// The next time this lecture takes place, as an RFC 3339 timestamp.
    #[schema(example = "2024-10-15T08:00:00Z")]
    next_occurrence_at: DateTime<Utc>,
    /// The upcoming occurrences of this lecture, in chronological order.
    ///
    /// The first element's `start_at` matches `next_occurrence_at`. The list is
    /// capped at whichever covers more events: the next 10 occurrences or those
    /// within a 14-day window.
    upcoming: Vec<UpcomingEvent>,
}

/// A campus event search result, carrying the full event-proposal pre-fill payload.
///
/// One entry per `events.csv` row, identified by the addition key: picking one
/// in a client pre-fills the event proposal form without a second round-trip,
/// so every CSV column rides along.
#[derive(Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct EventEntry {
    /// The `event_<hash>` addition key - the upsert identity shared by the
    /// `events.csv` row and its key-named images.
    #[schema(example = "event_9d02ddd940c43f87")]
    id: String,
    /// The display name of the result. Supports highlighting.
    #[schema(example = "\x19GARNIX\x17 Festival")]
    name: String,
    /// The description of the event.
    #[schema(example = "Open-air student festival.")]
    description: String,
    /// When the event starts, as an RFC 3339 timestamp.
    #[schema(example = "2026-06-15T14:00:00Z")]
    starts_at: DateTime<Utc>,
    /// When the event ends, as an RFC 3339 timestamp.
    #[schema(example = "2026-06-19T21:59:00Z")]
    ends_at: DateTime<Utc>,
    /// Latitude of the event location.
    #[schema(example = 48.262908)]
    lat: f64,
    /// Longitude of the event location.
    #[schema(example = 11.669102)]
    lon: f64,
    /// The `TUMonline` org id of the organising organisation.
    #[schema(example = 51897)]
    organising_org_id: i32,
    /// The `/cdn/thumb/…` delivery path of the event image.
    #[schema(example = "/cdn/thumb/event_9d02ddd940c43f87_0.webp")]
    image: String,
    /// The author of the event image (CC-BY attribution).
    #[schema(example = "Studentische Vertretung TUM")]
    image_author: String,
    /// Crop offset of the thumbnail image: pixels to shift the crop window
    /// along the image's longer axis. `0` when unset, so a client can recover the crop.
    #[schema(example = 14)]
    image_thumb_offset: i32,
    /// Crop offset of the header image: pixels to shift the crop window
    /// along the image's longer axis. `0` when unset, so a client can recover the crop.
    #[schema(example = 257)]
    image_header_offset: i32,
}

/// A Nominatim address search result.
///
/// Unlike a [`LocationEntry`], an address is not a `NavigaTUM` entity: it has no
/// canonical `/{type}/{id}` route, and its `addresstype` is an open Nominatim
/// vocabulary rather than the closed [`LocationEntryType`] set.
#[derive(Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct AddressEntry {
    /// The id of the address, derived from the OSM id.
    #[schema(example = "osm_182663548")]
    id: String,
    /// The raw Nominatim `addresstype` (e.g. `road` or `suburb`).
    #[schema(example = "road")]
    addresstype: String,
    /// The display name of the result.
    #[schema(example = "Boltzmannstraße")]
    name: String,
    /// Subtext to show below the search result.
    ///
    /// Contains the serialised address.
    #[schema(example = "Boltzmannstraße, Garching bei München")]
    subtext: String,
}

/// A section of `NavigaTUM` entity results (sites, buildings, rooms, POIs).
#[derive(Serialize, Clone, utoipa::ToSchema)]
pub struct LocationSection {
    entries: Vec<LocationEntry>,
    /// A recommendation how many of the entries should be displayed by default.
    ///
    /// The number is usually from `0`..`5`.
    /// More results might be displayed when clicking "expand".
    #[schema(example = 4)]
    n_visible: usize,
    /// The estimated (not exact) number of hits for that query
    #[serde(rename = "estimatedTotalHits")]
    #[schema(example = 6)]
    estimated_total_hits: usize,
}

/// A section of lecture results.
#[derive(Serialize, Clone, utoipa::ToSchema)]
pub struct LectureSection {
    entries: Vec<LectureEntry>,
    /// A recommendation how many of the entries should be displayed by default.
    ///
    /// The number is usually from `0`..`5`.
    /// More results might be displayed when clicking "expand".
    #[schema(example = 4)]
    n_visible: usize,
    /// The estimated (not exact) number of hits for that query
    #[serde(rename = "estimatedTotalHits")]
    #[schema(example = 6)]
    estimated_total_hits: usize,
}

/// A section of campus event results.
#[derive(Serialize, Clone, utoipa::ToSchema)]
pub struct EventSection {
    entries: Vec<EventEntry>,
    /// A recommendation how many of the entries should be displayed by default.
    ///
    /// The number is usually from `0`..`5`.
    /// More results might be displayed when clicking "expand".
    #[schema(example = 4)]
    n_visible: usize,
    /// The estimated (not exact) number of hits for that query
    #[serde(rename = "estimatedTotalHits")]
    #[schema(example = 6)]
    estimated_total_hits: usize,
}

/// A section of Nominatim address results.
#[derive(Serialize, Clone, utoipa::ToSchema)]
pub struct AddressSection {
    entries: Vec<AddressEntry>,
    /// A recommendation how many of the entries should be displayed by default.
    ///
    /// The number is usually from `0`..`5`.
    /// More results might be displayed when clicking "expand".
    #[schema(example = 4)]
    n_visible: usize,
    /// The estimated (not exact) number of hits for that query
    #[serde(rename = "estimatedTotalHits")]
    #[schema(example = 6)]
    estimated_total_hits: usize,
}

/// One section of search results, grouped by facet.
///
/// The `facet` is the discriminator: the four entity facets (sites, buildings,
/// rooms, and POIs) carry [`LocationEntry`]s with the room-id formatting fields,
/// the addresses facet carries [`AddressEntry`]s with their open Nominatim
/// `addresstype`, the lectures facet carries [`LectureEntry`]s with their
/// bilingual titles and upcoming occurrences, and the events facet carries
/// [`EventEntry`]s with the event-proposal pre-fill payload. Tagging the section
/// by `facet` means a consumer narrows once on the discriminator it already
/// needs for the section header and then sees exactly the entry shape that
/// facet carries - instead of a redundant per-entry `kind` repeated on every
/// hit.
#[derive(Serialize, Clone, utoipa::ToSchema)]
#[serde(tag = "facet", rename_all = "snake_case")]
pub enum ResultsSection {
    Sites(LocationSection),
    Buildings(LocationSection),
    Rooms(LocationSection),
    Pois(LocationSection),
    Addresses(AddressSection),
    Lectures(LectureSection),
    Events(EventSection),
}

impl ResultsSection {
    /// The facet this section groups.
    pub(crate) fn facet(&self) -> ResultFacet {
        match self {
            Self::Sites(_) => ResultFacet::Sites,
            Self::Buildings(_) => ResultFacet::Buildings,
            Self::Rooms(_) => ResultFacet::Rooms,
            Self::Pois(_) => ResultFacet::Pois,
            Self::Addresses(_) => ResultFacet::Addresses,
            Self::Lectures(_) => ResultFacet::Lectures,
            Self::Events(_) => ResultFacet::Events,
        }
    }
}

impl Debug for ResultsSection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sites(b) | Self::Buildings(b) | Self::Rooms(b) | Self::Pois(b) => f
                .debug_tuple(&format!("{:?}", self.facet()))
                .field(&TruncatedEntries(&b.entries))
                .finish(),
            Self::Addresses(b) => f
                .debug_tuple("Addresses")
                .field(&TruncatedEntries(&b.entries))
                .finish(),
            Self::Lectures(b) => f
                .debug_tuple("Lectures")
                .field(&TruncatedEntries(&b.entries))
                .finish(),
            Self::Events(b) => f
                .debug_tuple("Events")
                .field(&TruncatedEntries(&b.entries))
                .finish(),
        }
    }
}

/// Renders at most the first four entries of a section, eliding the tail, to
/// keep `Debug` output (used in tracing) from dumping large result sets.
struct TruncatedEntries<'a, E>(&'a [E]);

impl<E: Debug> Debug for TruncatedEntries<'_, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut base = f.debug_set();
        for entry in self.0.iter().take(4) {
            base.entry(entry);
        }
        if self.0.len() > 3 {
            base.entry(&"...");
        }
        base.finish()
    }
}

/// Section accessors used by the search tests, which reach for a particular
/// facet's section and its concretely-typed entries. Production code matches on
/// the variant directly, so these are test-only.
#[cfg(test)]
impl ResultsSection {
    /// The rooms section's body, if this is the rooms section.
    fn rooms(&self) -> Option<&LocationSection> {
        match self {
            Self::Rooms(s) => Some(s),
            _ => None,
        }
    }
    /// The lectures section's body, if this is the lectures section.
    fn lectures(&self) -> Option<&LectureSection> {
        match self {
            Self::Lectures(s) => Some(s),
            _ => None,
        }
    }
    /// The events section's body, if this is the events section.
    fn events(&self) -> Option<&EventSection> {
        match self {
            Self::Events(s) => Some(s),
            _ => None,
        }
    }
    /// Whether this section has no entries.
    fn is_empty(&self) -> bool {
        match self {
            Self::Sites(b) | Self::Buildings(b) | Self::Rooms(b) | Self::Pois(b) => {
                b.entries.is_empty()
            }
            Self::Addresses(b) => b.entries.is_empty(),
            Self::Lectures(b) => b.entries.is_empty(),
            Self::Events(b) => b.entries.is_empty(),
        }
    }
    /// The ids of every entry in this section, regardless of facet.
    fn entry_ids(&self) -> Vec<&str> {
        match self {
            Self::Sites(b) | Self::Buildings(b) | Self::Rooms(b) | Self::Pois(b) => {
                b.entries.iter().map(|e| e.id.as_str()).collect()
            }
            Self::Addresses(b) => b.entries.iter().map(|e| e.id.as_str()).collect(),
            Self::Lectures(b) => b.entries.iter().map(|e| e.id.as_str()).collect(),
            Self::Events(b) => b.entries.iter().map(|e| e.id.as_str()).collect(),
        }
    }
}

#[tracing::instrument]
pub async fn address_search(q: &str) -> LimitedVec<ResultsSection> {
    let results = match Nominatim::address_search(q).await {
        Ok(r) => r.0,
        Err(e) => {
            error!(error = ?e, "Error searching for addresses");
            return LimitedVec(vec![]);
        }
    };
    let num_results = results.len();
    let section = ResultsSection::Addresses(AddressSection {
        entries: results
            .into_iter()
            .map(|r| {
                let subtext = r.address.serialise();
                AddressEntry {
                    id: format!("osm_{}", r.osm_id),
                    addresstype: r.address_type,
                    name: r.address.road.unwrap_or(r.name),
                    subtext,
                }
            })
            .collect(),
        n_visible: num_results.min(15),
        estimated_total_hits: num_results,
    });
    LimitedVec::from(vec![section])
}

#[tracing::instrument(skip(client))]
pub async fn do_geoentry_search(
    client: &Client,
    q: &str,
    limits: Limits,
    formatting_config: FormattingConfig,
    filter: String,
    sorting: Vec<String>,
) -> LimitedVec<ResultsSection> {
    let parsed_input = ParsedQuery::from(q);

    let meili_query = parsed_input
        .tokens
        .clone()
        .into_iter()
        .map(|s| match s {
            TextToken::Text(t) => t,
            TextToken::SplittableText((t1, t2)) => format!("{t1} {t2} {t1}{t2}"),
        })
        .collect::<Vec<String>>()
        .join(" ");
    let mut request =
        GeoEntryQuery::from((client, meili_query.clone(), &limits, &formatting_config));
    for sort in &sorting {
        request.with_sorting(sort);
    }
    if !filter.is_empty() {
        request.with_filtering(&filter);
    }

    let response = match request.execute().await {
        Ok(response) => response,
        Err(e) => {
            error!(error = ?e, "Error searching for results");
            return LimitedVec(vec![]);
        }
    };
    let highlight_ctx = highlight::HighlightContext {
        query: &meili_query,
        pre: &formatting_config.highlighting.pre,
        post: &formatting_config.highlighting.post,
    };
    let merger::MergedSections {
        sites: section_sites,
        buildings: section_buildings,
        rooms: mut section_rooms,
        pois: section_pois,
        lectures: section_lectures,
        events: section_events,
        facet_order,
    } = merger::merge_search_results(
        &limits,
        &response.hits,
        response.facet_distribution.as_ref(),
        &highlight_ctx,
    );
    let visitor = formatter::RoomVisitor::from((parsed_input, formatting_config));
    section_rooms
        .entries
        .iter_mut()
        .for_each(|r| visitor.visit(r));

    // Order: non-empty facets first, in the order they first appeared in the
    // ranked Meilisearch hits (so a facet whose top hit is more relevant
    // ranks above one whose top hit is weaker). Empty sections trail at the
    // end so the caller can still observe `estimated_total_hits`.
    let mut sites_opt = Some(ResultsSection::Sites(section_sites));
    let mut buildings_opt = Some(ResultsSection::Buildings(section_buildings));
    let mut rooms_opt = Some(ResultsSection::Rooms(section_rooms));
    let mut pois_opt = Some(ResultsSection::Pois(section_pois));
    let mut lectures_opt = Some(ResultsSection::Lectures(section_lectures));
    // Address precedent for a default-disabled facet: the section only exists
    // when its query ran, so disabled requests keep their pre-facet shape.
    let mut events_opt =
        (limits.events_count > 0).then_some(ResultsSection::Events(section_events));

    let mut sections: Vec<ResultsSection> = Vec::with_capacity(ResultFacet::COUNT - 1);
    for facet in &facet_order {
        let taken = match facet {
            ResultFacet::Sites => sites_opt.take(),
            ResultFacet::Buildings => buildings_opt.take(),
            ResultFacet::Rooms => rooms_opt.take(),
            ResultFacet::Pois => pois_opt.take(),
            ResultFacet::Lectures => lectures_opt.take(),
            ResultFacet::Events => events_opt.take(),
            ResultFacet::Addresses => None,
        };
        if let Some(s) = taken {
            sections.push(s);
        }
    }
    for trailing in [
        sites_opt,
        buildings_opt,
        rooms_opt,
        pois_opt,
        lectures_opt,
        events_opt,
    ]
    .into_iter()
    .flatten()
    {
        sections.push(trailing);
    }
    LimitedVec(sections)
}

#[cfg(test)]
mod test {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::panic_in_result_fn,
        clippy::absolute_paths,
        reason = "tests assert via panic/unwrap and reference absolute paths to fixtures"
    )]
    use std::fmt::{self, Display, Formatter};

    use strum::EnumCount as _;

    use super::*;
    use crate::routes::search::{CroppingMode, Highlighting, ParsedIdMode};
    use crate::setup::tests::{MeiliSearchTestContainer, PostgresTestContainer};

    #[derive(serde::Deserialize)]
    struct TestQuery {
        target: String,
        query: String,
        among: Option<usize>,
        comment: Option<String>,
    }

    impl TestQuery {
        fn load_good() -> Vec<Self> {
            serde_yaml::from_str(include_str!("test-queries.good.yaml")).unwrap()
        }
        fn load_bad() -> Vec<Self> {
            serde_yaml::from_str(include_str!("test-queries.bad.yaml")).unwrap()
        }
        fn actual_matches_among(&self, actual: &[ResultsSection]) -> bool {
            let among = self.among.unwrap_or(1);
            let mut acceptable_range = actual
                .iter()
                .flat_map(ResultsSection::entry_ids)
                .take(among);
            acceptable_range.any(|id| id == self.target.as_str())
        }
        async fn search(&self, client: &Client) -> Vec<ResultsSection> {
            do_geoentry_search(
                client,
                &self.query,
                Limits::default(),
                FormattingConfig::default(),
                String::new(),
                vec![],
            )
            .await
            .0
        }
    }
    impl Display for TestQuery {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "'{query}' should get '{target}' in {among} results",
                query = self.query,
                target = self.target,
                among = self.among.unwrap_or(1),
            )?;
            if let Some(comment) = &self.comment {
                write!(f, " # {comment}")?;
            }
            Ok(())
        }
    }
    #[tokio::test]
    #[ignore = "flaky over time and low signal"]
    #[tracing_test::traced_test]
    async fn test_good_queries() {
        let ms = MeiliSearchTestContainer::new().await;
        ms.load_data_retrying().await;
        for query in TestQuery::load_good() {
            let actual = query.search(&ms.client).await;

            // Fail fast: a "good" query must return something; empty results usually mean a broken index/search.
            assert!(
                !actual.is_empty(),
                "{query}\n\
                Expected at least one results section, but got none"
            );

            assert!(
                query.actual_matches_among(&actual),
                "{query}\n\
                Since it can't, please move it to .bad list"
            );

            // Redact `estimatedTotalHits` to reduce snapshot churn.
            insta::with_settings!({
                info => &format!("{query}"),
                description => query.comment.unwrap_or_default(),
            }, {
                insta::assert_yaml_snapshot!(actual, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
            });
        }
    }

    #[tokio::test]
    #[ignore = "flaky over time and low signal"]
    #[tracing_test::traced_test]
    async fn test_bad_queries() {
        let ms = MeiliSearchTestContainer::new().await;
        ms.load_data_retrying().await;
        for query in TestQuery::load_bad() {
            let actual = query.search(&ms.client).await;

            // "Bad" queries may return results, but must not surface the `target` in the first `among` results.
            assert!(
                !query.actual_matches_among(&actual),
                "{query}\n\
                Since it can, please move it to .good list"
            );

            // Redact `estimatedTotalHits` to reduce snapshot churn.
            insta::with_settings!({
                info => &format!("{query}"),
                description => query.comment.unwrap_or_default(),
            }, {
                insta::assert_yaml_snapshot!(actual, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
            });
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_cropping_full_shows_full_building_names() {
        let ms = MeiliSearchTestContainer::new().await;
        ms.load_data_retrying().await;

        // Verify `CroppingMode` changes output; fail fast on missing Rooms/entries and snapshot both variants.
        let query = "N-1406";

        // Search with cropping enabled.
        let config_cropping = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results_cropping = do_geoentry_search(
            &ms.client,
            query,
            Limits::default(),
            config_cropping,
            String::new(),
            vec![],
        )
        .await;

        // Search with cropping disabled.
        let config_no_cropping = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Full,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results_no_cropping = do_geoentry_search(
            &ms.client,
            query,
            Limits::default(),
            config_no_cropping,
            String::new(),
            vec![],
        )
        .await;

        // Extract Rooms; fail fast to avoid silent no-ops.
        let rooms_with_crop = results_cropping
            .0
            .iter()
            .find_map(ResultsSection::rooms)
            .expect("Expected a Rooms section for cropping=CROP test query");

        let rooms_without_crop = results_no_cropping
            .0
            .iter()
            .find_map(ResultsSection::rooms)
            .expect("Expected a Rooms section for cropping=FULL test query");

        assert!(
            !rooms_with_crop.entries.is_empty(),
            "Expected at least one room entry for cropping=CROP test query"
        );
        assert!(
            !rooms_without_crop.entries.is_empty(),
            "Expected at least one room entry for cropping=FULL test query"
        );

        // Compare deterministically by sorting by `id` so ranking changes don't flap.
        let mut ids_cropped: Vec<(&str, Option<&str>)> = rooms_with_crop
            .entries
            .iter()
            .map(|e| (e.id.as_str(), e.parsed_id.as_deref()))
            .collect();
        ids_cropped.sort_by(|a, b| a.0.cmp(b.0));

        let mut ids_full: Vec<(&str, Option<&str>)> = rooms_without_crop
            .entries
            .iter()
            .map(|e| (e.id.as_str(), e.parsed_id.as_deref()))
            .collect();
        ids_full.sort_by(|a, b| a.0.cmp(b.0));

        // For the same `id`, `cropping=FULL` must not produce a shorter `parsed_id` than `cropping=CROP`.
        for ((id_c, pid_c), (id_f, pid_f)) in ids_cropped.iter().zip(ids_full.iter()) {
            assert_eq!(id_c, id_f, "Expected comparable entries when sorting by id");
            if let (Some(c), Some(f)) = (pid_c.as_ref(), pid_f.as_ref()) {
                assert!(
                    f.chars().count() >= c.chars().count(),
                    "Expected cropping=FULL parsed_id to be >= cropping=CROP parsed_id length for id={id_c}"
                );
            }
        }

        // Snapshot both outputs; redact `estimatedTotalHits`.
        insta::with_settings!({
            info => &"cropping=crop",
            description => format!("Query: {query}"),
        }, {
            insta::assert_yaml_snapshot!(results_cropping.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });

        insta::with_settings!({
            info => &"cropping=full",
            description => format!("Query: {query}"),
        }, {
            insta::assert_yaml_snapshot!(results_no_cropping.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_both_flags_work_together() {
        let ms = MeiliSearchTestContainer::new().await;
        ms.load_data_retrying().await;

        // Validate `ParsedIdMode::Roomfinder`: `parsed_id` should look like an arch id (contains '@'); snapshot output.
        let config = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Full,
            parsed_id: ParsedIdMode::Roomfinder,
        };

        // Use a canonical query from the good list to avoid accidental no-op.
        let results = do_geoentry_search(
            &ms.client,
            "N-1406",
            Limits::default(),
            config,
            String::new(),
            vec![],
        )
        .await;

        let room_section = results
            .0
            .iter()
            .find_map(ResultsSection::rooms)
            .expect("Expected a Rooms section for Roomfinder mode test");

        assert!(
            !room_section.entries.is_empty(),
            "Expected at least one room entry for Roomfinder mode test"
        );

        for entry in &room_section.entries {
            let pid = entry
                .parsed_id
                .as_deref()
                .expect("Expected parsed_id to be present in Roomfinder mode");

            assert!(
                pid.contains('@'),
                "Expected Roomfinder parsed_id to contain '@' (arch_id@building_id), got: {pid}"
            );
        }

        insta::with_settings!({
            info => &"parsed_id=roomfinder,cropping=full",
            description => "Query: N-1406",
        }, {
            insta::assert_yaml_snapshot!(results.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_custom_highlighting_with_formatting() {
        let ms = MeiliSearchTestContainer::new().await;
        ms.load_data_retrying().await;

        // Validate custom highlighting end-to-end; "MW1801" is a canonical query that tends to trigger highlighting.
        let query = "MW1801";

        let config = FormattingConfig {
            highlighting: Highlighting {
                pre: "<em>".to_string(),
                post: "</em>".to_string(),
            },
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results = do_geoentry_search(
            &ms.client,
            query,
            Limits::default(),
            config,
            String::new(),
            vec![],
        )
        .await;

        let room_section = results
            .0
            .iter()
            .find_map(ResultsSection::rooms)
            .expect("Expected a Rooms section for highlighting test");

        assert!(
            !room_section.entries.is_empty(),
            "Expected at least one room entry for highlighting test"
        );

        let has_custom_highlighting = room_section.entries.iter().any(|e| {
            let in_parsed_id = e
                .parsed_id
                .as_deref()
                .is_some_and(|p| p.contains("<em>") || p.contains("</em>"));

            let in_name = e.name.contains("<em>") || e.name.contains("</em>");

            in_parsed_id || in_name
        });

        assert!(
            has_custom_highlighting,
            "Expected custom highlighting markers to appear in results for query '{query}'"
        );

        insta::with_settings!({
            info => &"highlighting=<em>",
            description => format!("Query: {query}"),
        }, {
            insta::assert_yaml_snapshot!(results.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_building_formats() {
        let ms = MeiliSearchTestContainer::new().await;
        ms.load_data_retrying().await;

        let config_prefixed = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let config_roomfinder = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Roomfinder,
        };

        // Canonical queries that exercise parsed_id mode behavior.
        //
        // We intentionally *don't* assert concrete building prefixes here, because:
        // - whether `parsed_id` is present and/or prefixed depends on query parsing + hit metadata
        // - search ranking/index changes can legitimately change which rooms appear in the top results
        //
        // Instead, we assert the semantic contract:
        // - `ParsedIdMode::Prefixed`: if `parsed_id` is present, it must *not* be raw Roomfinder format
        // - `ParsedIdMode::Roomfinder`: at least one `parsed_id` must contain '@'
        let test_queries = vec!["MW1801", "MI5601", "PH5101"];

        for query in test_queries {
            let results_prefixed = do_geoentry_search(
                &ms.client,
                query,
                Limits::default(),
                config_prefixed.clone(),
                String::new(),
                vec![],
            )
            .await;

            let results_roomfinder = do_geoentry_search(
                &ms.client,
                query,
                Limits::default(),
                config_roomfinder.clone(),
                String::new(),
                vec![],
            )
            .await;

            let rooms_prefixed = results_prefixed
                .0
                .iter()
                .find_map(ResultsSection::rooms)
                .expect("Expected a Rooms section for prefixed mode");

            assert!(
                !rooms_prefixed.entries.is_empty(),
                "Expected at least one room entry for prefixed mode query '{query}'"
            );

            // In prefixed mode, we don't require `parsed_id` to always be present for every result
            // (it depends on query parsing and hit metadata). But if it *is* present, it must not
            // look like a raw Roomfinder arch name (arch_id@building_id).
            for entry in &rooms_prefixed.entries {
                if let Some(pid) = entry.parsed_id.as_deref() {
                    assert!(
                        !pid.contains('@'),
                        "Expected prefixed parsed_id to not contain '@' for query '{query}', got: {pid}"
                    );
                }
            }

            let rooms_roomfinder = results_roomfinder
                .0
                .iter()
                .find_map(ResultsSection::rooms)
                .expect("Expected a Rooms section for roomfinder mode");

            assert!(
                !rooms_roomfinder.entries.is_empty(),
                "Expected at least one room entry for roomfinder mode query '{query}'"
            );

            // In Roomfinder mode, `parsed_id` should be the raw `arch_name` (contains '@').
            // Do not assume every entry has it, but require that at least one does.
            let has_raw_archname_format = rooms_roomfinder
                .entries
                .iter()
                .any(|e| e.parsed_id.as_deref().is_some_and(|p| p.contains('@')));

            assert!(
                has_raw_archname_format,
                "Expected at least one Roomfinder parsed_id to contain '@' for query '{query}'"
            );

            // Snapshot only stable fields to reduce brittleness across ranking/index changes.
            insta::with_settings!({
                info => &"parsed_id=prefixed",
                description => format!("Query: {query}"),
            }, {
                insta::assert_yaml_snapshot!(results_prefixed.0, {
                    ".**.estimatedTotalHits" => "[estimatedTotalHits]",
                });
            });

            insta::with_settings!({
                info => &"parsed_id=roomfinder",
                description => format!("Query: {query}"),
            }, {
                insta::assert_yaml_snapshot!(results_roomfinder.0, {
                    ".**.estimatedTotalHits" => "[estimatedTotalHits]",
                });
            });
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_cropping_behavior_for_1010() {
        let ms = MeiliSearchTestContainer::new().await;
        ms.load_data_retrying().await;

        // Verify `CroppingMode` changes output; compare `parsed_id` lengths by sorted `id` (Full must not be shorter).
        let query = "1010 znn";

        // 1. Search with cropping enabled
        let config_cropped = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results_cropped = do_geoentry_search(
            &ms.client,
            query,
            Limits::default(),
            config_cropped,
            String::new(),
            vec![],
        )
        .await;

        // 2. Search with cropping disabled
        let config_full = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Full,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results_full = do_geoentry_search(
            &ms.client,
            query,
            Limits::default(),
            config_full,
            String::new(),
            vec![],
        )
        .await;

        let rooms_cropped = results_cropped
            .0
            .iter()
            .find_map(ResultsSection::rooms)
            .expect("Expected a Rooms section for cropping=CROP");

        let rooms_full = results_full
            .0
            .iter()
            .find_map(ResultsSection::rooms)
            .expect("Expected a Rooms section for cropping=FULL");

        assert!(
            !rooms_cropped.entries.is_empty(),
            "Expected at least one room entry for cropping=CROP query '{query}'"
        );
        assert!(
            !rooms_full.entries.is_empty(),
            "Expected at least one room entry for cropping=FULL query '{query}'"
        );

        // Deterministic comparison: sort by id and compare overlapping entries.
        let mut cropped: Vec<(&str, Option<&str>)> = rooms_cropped
            .entries
            .iter()
            .map(|e| (e.id.as_str(), e.parsed_id.as_deref()))
            .collect();
        cropped.sort_by(|a, b| a.0.cmp(b.0));

        let mut full: Vec<(&str, Option<&str>)> = rooms_full
            .entries
            .iter()
            .map(|e| (e.id.as_str(), e.parsed_id.as_deref()))
            .collect();
        full.sort_by(|a, b| a.0.cmp(b.0));

        // Assert FULL is never shorter than CROP for the same `id`.
        for ((id_c, pid_c), (id_f, pid_f)) in cropped.iter().zip(full.iter()) {
            assert_eq!(id_c, id_f, "Expected comparable entries when sorting by id");
            if let (Some(c), Some(f)) = (pid_c.as_ref(), pid_f.as_ref()) {
                assert!(
                    f.chars().count() >= c.chars().count(),
                    "Expected cropping=FULL parsed_id to be >= cropping=CROP parsed_id length for id={id_c}"
                );
            }
        }

        insta::with_settings!({
            info => &"cropping=crop",
            description => format!("Query: {query}"),
        }, {
            insta::assert_yaml_snapshot!(results_cropped.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });

        insta::with_settings!({
            info => &"cropping=full",
            description => format!("Query: {query}"),
        }, {
            insta::assert_yaml_snapshot!(results_full.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_lecture_facet_query() {
        let ms = MeiliSearchTestContainer::new().await;

        // The lecture facet is normally derived from the calendar table by the
        // refresh task. Here we upsert a single known lecture document directly
        // so the snapshot is deterministic and independent of live data: the
        // synthetic title cannot collide with any geo entry, so the query
        // surfaces only the Lectures section.
        let lecture = serde_json::json!({
            "ms_id": "lecture_testfixture0001",
            "facet": "lecture",
            "type_common_name": "Vorlesung",
            "title_de": "Grundlagen der Navigatumlehre",
            "title_en": "Foundations of Navigatum Teaching",
            "name": "Grundlagen der Navigatumlehre",
            "rank": 0,
            "parent_building_names": ["Maschinenwesen (MW)"],
            "parent_keywords": ["mw", "garching"],
            "next_occurrence_at": "2024-10-15T08:00:00Z",
            "upcoming": [
                {
                    "start_at": "2024-10-15T08:00:00Z",
                    "end_at": "2024-10-15T10:00:00Z",
                    "room_code": "5606.EG.011",
                    "room_name": "Testhörsaal",
                },
                {
                    "start_at": "2024-10-22T08:00:00Z",
                    "end_at": "2024-10-22T10:00:00Z",
                    "room_code": "5606.EG.011",
                    "room_name": "Testhörsaal",
                },
            ],
        });
        let task = ms
            .client
            .index("entries")
            .add_documents(&[lecture], Some("ms_id"))
            .await
            .unwrap()
            .wait_for_completion(&ms.client, None, Some(std::time::Duration::from_secs(30)))
            .await
            .unwrap();
        assert!(
            matches!(task, meilisearch_sdk::tasks::Task::Succeeded { .. }),
            "lecture document upsert should succeed, got {task:?}"
        );

        let results = do_geoentry_search(
            &ms.client,
            "Navigatumlehre",
            Limits::default(),
            FormattingConfig::default(),
            String::new(),
            vec![],
        )
        .await;

        // One section per federated facet, minus events (default-disabled here);
        // the handler appends the address section.
        assert_eq!(results.0.len(), ResultFacet::COUNT - 2);

        let lectures = results
            .0
            .iter()
            .find_map(ResultsSection::lectures)
            .expect("expected a Lectures section for a lecture-title query");
        let top = lectures
            .entries
            .first()
            .expect("expected at least one lecture entry");
        assert_eq!(top.id, "lecture_testfixture0001");
        assert_eq!(top.title_de, "Grundlagen der Navigatumlehre");
        assert_eq!(top.title_en, "Foundations of Navigatum Teaching");
        assert_eq!(
            top.next_occurrence_at,
            "2024-10-15T08:00:00Z".parse::<DateTime<Utc>>().unwrap()
        );
        // The human `stp_type` label is surfaced as the subtext.
        assert_eq!(top.subtext, "Vorlesung");
        // The upcoming occurrences ride along, room names resolved, with the
        // first element's start matching `next_occurrence_at`.
        let upcoming = &top.upcoming;
        assert_eq!(upcoming.len(), 2);
        let first = upcoming.first().unwrap();
        assert_eq!(first.start_at, top.next_occurrence_at);
        assert_eq!(first.room_code, "5606.EG.011");
        assert_eq!(first.room_name, "Testhörsaal");

        insta::with_settings!({
            info => &"q=Navigatumlehre",
            description => "lecture facet returns a bilingual hit with its next occurrence",
        }, {
            insta::assert_yaml_snapshot!(results.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });
    }

    /// The full document shape the data pipeline exports for one `events.csv` row.
    fn garnix_event_document() -> serde_json::Value {
        serde_json::json!({
            "ms_id": "event_9d02ddd940c43f87",
            "facet": "event",
            "name": "GARNIX Festival",
            "starts_at": "2026-06-15T14:00:00Z",
            "ends_at": "2026-06-19T21:59:00Z",
            "description": "Open-air student festival.",
            "organising_org_id": 51897,
            "image": "/cdn/thumb/event_9d02ddd940c43f87_0.webp",
            "image_author": "Studentische Vertretung TUM",
            "rank": 0,
            "_geo": {"lat": 48.262908, "lng": 11.669102},
        })
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_event_facet_returns_the_prefill_payload() {
        let ms = MeiliSearchTestContainer::new().await;
        let task = ms
            .client
            .index("entries")
            .add_documents(&[garnix_event_document()], Some("ms_id"))
            .await
            .unwrap()
            .wait_for_completion(&ms.client, None, Some(std::time::Duration::from_secs(30)))
            .await
            .unwrap();
        assert!(
            matches!(task, meilisearch_sdk::tasks::Task::Succeeded { .. }),
            "event document upsert should succeed, got {task:?}"
        );

        let limits = Limits {
            events_count: 5,
            ..Limits::default()
        };
        let results = do_geoentry_search(
            &ms.client,
            "garnix",
            limits,
            FormattingConfig::default(),
            String::new(),
            vec![],
        )
        .await;

        // One section per federated facet incl. events; the handler appends addresses.
        assert_eq!(results.0.len(), ResultFacet::COUNT - 1);

        let events = results
            .0
            .iter()
            .find_map(ResultsSection::events)
            .expect("expected an Events section for an event-name query");
        let top = events
            .entries
            .first()
            .expect("expected at least one event entry");
        // Everything the propose page needs to pre-fill the event form rides along.
        assert_eq!(top.id, "event_9d02ddd940c43f87");
        assert_eq!(top.name, "\u{19}GARNIX\u{17} Festival");
        assert_eq!(top.description, "Open-air student festival.");
        assert_eq!(
            top.starts_at,
            "2026-06-15T14:00:00Z".parse::<DateTime<Utc>>().unwrap()
        );
        assert_eq!(
            top.ends_at,
            "2026-06-19T21:59:00Z".parse::<DateTime<Utc>>().unwrap()
        );
        assert!((top.lat - 48.262908).abs() < f64::EPSILON);
        assert!((top.lon - 11.669102).abs() < f64::EPSILON);
        assert_eq!(top.organising_org_id, 51897);
        assert_eq!(top.image, "/cdn/thumb/event_9d02ddd940c43f87_0.webp");
        assert_eq!(top.image_author, "Studentische Vertretung TUM");
        // This document predates the crop offsets: a pre-existing index entry lacking
        // the fields must deserialize to 0, not fail.
        assert_eq!(top.image_thumb_offset, 0);
        assert_eq!(top.image_header_offset, 0);

        // The event surfaces only in its own section, never in the five geo/lecture ones.
        for section in &results.0 {
            if section.facet() != ResultFacet::Events {
                assert!(
                    !section.entry_ids().contains(&top.id.as_str()),
                    "event leaked into the {facet:?} section",
                    facet = section.facet()
                );
            }
        }

        insta::with_settings!({
            info => &"q=garnix, search_events=true",
            description => "the event facet returns the full proposal-form pre-fill payload",
        }, {
            insta::assert_yaml_snapshot!(results.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_event_image_crop_offsets_thread_through() {
        let ms = MeiliSearchTestContainer::new().await;
        let mut document = garnix_event_document();
        let fields = document.as_object_mut().expect("event document is a JSON object");
        fields.insert("image_thumb_offset".to_owned(), serde_json::json!(14));
        fields.insert("image_header_offset".to_owned(), serde_json::json!(257));
        let task = ms
            .client
            .index("entries")
            .add_documents(&[document], Some("ms_id"))
            .await
            .unwrap()
            .wait_for_completion(&ms.client, None, Some(std::time::Duration::from_secs(30)))
            .await
            .unwrap();
        assert!(
            matches!(task, meilisearch_sdk::tasks::Task::Succeeded { .. }),
            "event document upsert should succeed, got {task:?}"
        );

        let limits = Limits {
            events_count: 5,
            ..Limits::default()
        };
        let results = do_geoentry_search(
            &ms.client,
            "garnix",
            limits,
            FormattingConfig::default(),
            String::new(),
            vec![],
        )
        .await;

        let top = results
            .0
            .iter()
            .find_map(ResultsSection::events)
            .and_then(|events| events.entries.first())
            .expect("expected at least one event entry");
        assert_eq!(top.image_thumb_offset, 14);
        assert_eq!(top.image_header_offset, 257);
    }

    /// The event facet is default-disabled: without `search_events=true` or
    /// `type=event` (both encoded as a non-zero `events_count`), an indexed
    /// event must be invisible - no events section, no hits in other sections.
    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_event_facet_is_invisible_unless_enabled() {
        let ms = MeiliSearchTestContainer::new().await;
        let task = ms
            .client
            .index("entries")
            .add_documents(&[garnix_event_document()], Some("ms_id"))
            .await
            .unwrap()
            .wait_for_completion(&ms.client, None, Some(std::time::Duration::from_secs(30)))
            .await
            .unwrap();
        assert!(
            matches!(task, meilisearch_sdk::tasks::Task::Succeeded { .. }),
            "event document upsert should succeed, got {task:?}"
        );

        let results = do_geoentry_search(
            &ms.client,
            "garnix",
            Limits::default(),
            FormattingConfig::default(),
            String::new(),
            vec![],
        )
        .await;

        // The response keeps its pre-facet shape: the five always-on sections.
        assert_eq!(results.0.len(), ResultFacet::COUNT - 2);
        assert!(
            results.0.iter().find_map(ResultsSection::events).is_none(),
            "a disabled event facet must not produce an events section"
        );
        for section in &results.0 {
            assert!(
                !section.entry_ids().contains(&"event_9d02ddd940c43f87"),
                "event leaked into the {facet:?} section despite the facet being disabled",
                facet = section.facet()
            );
        }
    }

    /// `?type=event` end-to-end below the handler: `Limits::from` turns the
    /// filter into a non-zero `events_count` (unit-tested in `routes::search`),
    /// and the `facet IN ["event"]` user filter built from it must keep every
    /// other section empty even when a geo entry matches the query.
    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_event_type_filter_returns_only_events() {
        let ms = MeiliSearchTestContainer::new().await;
        // A building sharing the query token, to prove the type filter excludes it.
        let building = serde_json::json!({
            "ms_id": "building_testfixture0001",
            "facet": "building",
            "type": "building",
            "room_code": "GRX",
            "name": "GARNIX Pavillon",
            "type_common_name": "Gebäude",
            "rank": 100,
            "parent_building_names": [],
            "parent_keywords": ["garching"],
        });
        let task = ms
            .client
            .index("entries")
            .add_documents(&[building, garnix_event_document()], Some("ms_id"))
            .await
            .unwrap()
            .wait_for_completion(&ms.client, None, Some(std::time::Duration::from_secs(30)))
            .await
            .unwrap();
        assert!(
            matches!(task, meilisearch_sdk::tasks::Task::Succeeded { .. }),
            "fixture upsert should succeed, got {task:?}"
        );

        let limits = Limits {
            events_count: 5,
            ..Limits::default()
        };
        let results = do_geoentry_search(
            &ms.client,
            "garnix",
            limits,
            FormattingConfig::default(),
            r#"(facet IN ["event"])"#.to_string(),
            vec![],
        )
        .await;

        let events = results
            .0
            .iter()
            .find_map(ResultsSection::events)
            .expect("expected an Events section for a type=event query");
        assert_eq!(events.entries.len(), 1);
        assert_eq!(events.entries.first().unwrap().id, "event_9d02ddd940c43f87");
        for section in &results.0 {
            assert!(
                section.facet() == ResultFacet::Events || section.is_empty(),
                "type=event must keep the {facet:?} section empty",
                facet = section.facet()
            );
        }
    }

    /// Successive editions of a recurring event (submitted with new posters, so
    /// new keys) share a name and tie on every text ranking rule. The per-query
    /// `starts_at:desc` sort breaks the tie in favour of the newest edition -
    /// the one a proposer wants to start from. The older edition is inserted
    /// first so that, without the sort, insertion order would surface it on top
    /// and fail the test.
    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_newest_event_edition_ranks_first() {
        let ms = MeiliSearchTestContainer::new().await;
        let editions = [
            serde_json::json!({
                "ms_id": "event_17ddb108241f623c",
                "facet": "event",
                "name": "GARNIX Festival",
                "starts_at": "2025-06-16T14:00:00Z",
                "ends_at": "2025-06-20T21:59:00Z",
                "description": "Last year's edition.",
                "organising_org_id": 51897,
                "image": "/cdn/thumb/event_17ddb108241f623c_0.webp",
                "image_author": "Studentische Vertretung TUM",
                "rank": 0,
                "_geo": {"lat": 48.262908, "lng": 11.669102},
            }),
            garnix_event_document(),
        ];
        let task = ms
            .client
            .index("entries")
            .add_documents(&editions, Some("ms_id"))
            .await
            .unwrap()
            .wait_for_completion(&ms.client, None, Some(std::time::Duration::from_secs(30)))
            .await
            .unwrap();
        assert!(
            matches!(task, meilisearch_sdk::tasks::Task::Succeeded { .. }),
            "event document upsert should succeed, got {task:?}"
        );

        let limits = Limits {
            events_count: 5,
            ..Limits::default()
        };
        let results = do_geoentry_search(
            &ms.client,
            "garnix",
            limits,
            FormattingConfig::default(),
            String::new(),
            vec![],
        )
        .await;

        let events = results
            .0
            .iter()
            .find_map(ResultsSection::events)
            .expect("expected an Events section for an event-name query");
        let ids: Vec<&str> = events.entries.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(
            ids,
            ["event_9d02ddd940c43f87", "event_17ddb108241f623c"],
            "the newest edition must rank first"
        );

        insta::with_settings!({
            info => &"q=garnix, search_events=true",
            description => "same-name editions are sorted by starts_at descending",
        }, {
            insta::assert_yaml_snapshot!(results.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });
    }

    /// A lecture and a building whose names share query tokens, where the
    /// lecture is the *stronger* raw match (it matches both query tokens, the
    /// building only one). The 0.5 federation weight still demotes the lecture
    /// beneath the building, so the Buildings section ranks above the Lectures
    /// section in the merged output. Without the weight the lecture's higher raw
    /// score surfaces it first - so the test fails if the weight is dropped.
    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_lecture_deprioritised_below_geo_on_shared_tokens() {
        let ms = MeiliSearchTestContainer::new().await;

        // The building matches one query token; the lecture matches both, so its
        // raw `_rankingScore` is the higher of the two. The federation weight is
        // what tips the merged ranking back in the building's favour - this makes
        // the test fail (lecture on top) if the weight is ever dropped, rather
        // than passing on an incidental query-order tie-break.
        let building = serde_json::json!({
            "ms_id": "building_testfixture0001",
            "facet": "building",
            "type": "building",
            "room_code": "QRZ",
            "name": "Quantenrobotik",
            "type_common_name": "Gebäude",
            "rank": 100,
            "parent_building_names": [],
            "parent_keywords": ["qrz", "garching"],
        });
        let lecture = serde_json::json!({
            "ms_id": "lecture_testfixture0001",
            "facet": "lecture",
            "type_common_name": "Vorlesung",
            "title_de": "Quantenrobotik Praktikum",
            "title_en": "Quantum Robotics Lab",
            "name": "Quantenrobotik Praktikum",
            "rank": 100,
            "parent_building_names": [],
            "parent_keywords": ["garching"],
            "next_occurrence_at": "2024-10-15T08:00:00Z",
            "upcoming": [
                {
                    "start_at": "2024-10-15T08:00:00Z",
                    "end_at": "2024-10-15T10:00:00Z",
                    "room_code": "5606.EG.011",
                    "room_name": "Testhörsaal",
                },
            ],
        });
        let task = ms
            .client
            .index("entries")
            .add_documents(&[building, lecture], Some("ms_id"))
            .await
            .unwrap()
            .wait_for_completion(&ms.client, None, Some(std::time::Duration::from_secs(30)))
            .await
            .unwrap();
        assert!(
            matches!(task, meilisearch_sdk::tasks::Task::Succeeded { .. }),
            "fixture upsert should succeed, got {task:?}"
        );

        let results = do_geoentry_search(
            &ms.client,
            "Quantenrobotik Praktikum",
            Limits::default(),
            FormattingConfig::default(),
            String::new(),
            vec![],
        )
        .await;

        let position = |facet: ResultFacet| {
            results
                .0
                .iter()
                .position(|s| s.facet() == facet && !s.is_empty())
        };
        let buildings_at = position(ResultFacet::Buildings)
            .expect("expected a non-empty Buildings section for the shared-token query");
        let lectures_at = position(ResultFacet::Lectures)
            .expect("expected a non-empty Lectures section for the shared-token query");
        assert!(
            buildings_at < lectures_at,
            "the building must rank above the demoted lecture, \
             got buildings at {buildings_at} and lectures at {lectures_at}"
        );

        insta::with_settings!({
            info => &"q=Quantenrobotik Praktikum",
            description => "the 0.5 federation weight demotes the lecture below a building even when the lecture is the stronger raw match",
        }, {
            insta::assert_yaml_snapshot!(results.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });
    }

    /// Subset of a stored lecture document, used to assert the room-parent
    /// enrichment landed in the index.
    #[derive(serde::Deserialize)]
    struct IndexedLecture {
        parent_building_names: Vec<String>,
        parent_keywords: Vec<String>,
        upcoming: Vec<UpcomingEvent>,
    }

    /// End-to-end: seed the `calendar` table and a hosting room, run the real
    /// derivation task, and confirm the lecture it produces is searchable. This
    /// is the one test that exercises the actual pipeline (the SQL aggregation,
    /// the stable `ms_id`, the room-parent enrichment, and the stale cleanup)
    /// rather than a hand-injected document.
    #[tokio::test]
    #[tracing_test::traced_test]
    #[expect(
        clippy::too_many_lines,
        reason = "one end-to-end test seeds Postgres + Meilisearch, derives, and asserts across the whole pipeline"
    )]
    async fn test_lectures_derived_from_calendar_surface_in_search() {
        async fn insert_event(
            pool: &sqlx::PgPool,
            room_code: &str,
            id: i32,
            start: DateTime<Utc>,
            end: DateTime<Utc>,
        ) {
            // Same title/stp_type across rows so they collapse to one identity.
            sqlx::query(
                "INSERT INTO calendar \
                 (id, room_code, start_at, end_at, title_de, title_en, stp_type, entry_type, detailed_entry_type) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(id)
            .bind(room_code)
            .bind(start)
            .bind(end)
            .bind("Quantenfeldtheorie im Teststand")
            .bind("Quantum Field Theory on a Test Bench")
            .bind(Some("Vorlesung"))
            .bind("lecture")
            .bind("Vorlesung")
            .execute(pool)
            .await
            .unwrap();
        }

        let pg = PostgresTestContainer::new().await;
        let ms = MeiliSearchTestContainer::new().await;
        let entries = ms.client.index("entries");

        // `calendar.room_code` is a foreign key into `en` (which references `de`),
        // so the hosting room must exist in both before any calendar row can.
        // `name`/`type`/`lat`/`lon` are columns generated from `data`, so only
        // `key` and `data` are insertable - mirroring the real loader.
        let room_code = "5606.EG.011";
        let room_data = serde_json::json!({
            "name": "Testhörsaal",
            "type": "room",
            "type_common_name": "Hörsaal",
            "coords": { "lat": 48.0, "lon": 11.0, "source": "navigatum" },
        })
        .to_string();
        sqlx::query("INSERT INTO de (key, data) VALUES ($1, $2::jsonb)")
            .bind(room_code)
            .bind(&room_data)
            .execute(&pg.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO en (key, data) VALUES ($1, $2::jsonb)")
            .bind(room_code)
            .bind(&room_data)
            .execute(&pg.pool)
            .await
            .unwrap();

        // The geo room document the lecture inherits its parent context from.
        // Its name cannot collide with the lecture title, so it never shows up
        // in the lecture-title query and the snapshot stays deterministic.
        let room_doc = serde_json::json!({
            "ms_id": "5606-EG-011",
            "facet": "room",
            "type": "room",
            "room_code": room_code,
            "name": "Testhörsaal",
            "type_common_name": "Hörsaal",
            "rank": 100,
            "parent_building_names": ["Physik (PH)"],
            "parent_keywords": ["ph", "garching"],
        });
        entries
            .add_documents(&[room_doc], Some("ms_id"))
            .await
            .unwrap()
            .wait_for_completion(&ms.client, None, Some(std::time::Duration::from_secs(30)))
            .await
            .unwrap();

        let now = Utc::now().timestamp();
        let at = |secs: i64| DateTime::from_timestamp(now + secs, 0).unwrap();
        // Two future occurrences (the earliest at +1h) and one already-ended row
        // that the `end_at >= NOW()` filter must drop from `next_occurrence_at`.
        insert_event(&pg.pool, room_code, 1, at(3_600), at(7_200)).await;
        insert_event(&pg.pool, room_code, 2, at(10_800), at(14_400)).await;
        insert_event(&pg.pool, room_code, 3, at(-7_200), at(-3_600)).await;

        crate::refresh::lectures::refresh_once(&pg.pool, &ms.client)
            .await
            .unwrap();

        let results = do_geoentry_search(
            &ms.client,
            "Quantenfeldtheorie",
            Limits::default(),
            FormattingConfig::default(),
            String::new(),
            vec![],
        )
        .await;

        let lectures = results
            .0
            .iter()
            .find_map(ResultsSection::lectures)
            .expect("the derived lecture should surface in a Lectures section");
        assert_eq!(
            lectures.entries.len(),
            1,
            "the two future occurrences collapse into one lecture identity"
        );
        let top = lectures.entries.first().unwrap();
        assert!(top.id.starts_with("lecture_"), "got id {}", top.id);
        assert_eq!(top.title_de, "Quantenfeldtheorie im Teststand");
        assert_eq!(top.title_en, "Quantum Field Theory on a Test Bench");
        assert_eq!(top.subtext, "Vorlesung");
        // The earliest *future* occurrence wins; the past row is excluded.
        assert_eq!(top.next_occurrence_at, at(3_600));
        // Both future occurrences are materialised in chronological order with
        // the hosting room's display name resolved; the past row is dropped and
        // the first occurrence agrees with `next_occurrence_at`.
        let upcoming = &top.upcoming;
        assert_eq!(
            upcoming.len(),
            2,
            "the two future rows surface; the already-ended row is dropped"
        );
        let first = upcoming.first().unwrap();
        assert_eq!(first.start_at, at(3_600));
        assert_eq!(first.end_at, at(7_200));
        assert_eq!(first.room_code, room_code);
        assert_eq!(first.room_name, "Testhörsaal");
        assert_eq!(upcoming.get(1).unwrap().start_at, at(10_800));

        // The stored document inherits the hosting room's parent context, so
        // queries like "Quantenfeldtheorie PH" can find it, and it carries the
        // same resolved upcoming occurrences.
        let stored = meilisearch_sdk::documents::DocumentsQuery::new(&entries)
            .with_filter("facet = \"lecture\"")
            .execute::<IndexedLecture>()
            .await
            .unwrap();
        assert_eq!(stored.results.len(), 1);
        let stored = stored.results.first().unwrap();
        assert_eq!(stored.parent_building_names, ["Physik (PH)"]);
        assert_eq!(stored.parent_keywords, ["ph", "garching"]);
        assert_eq!(stored.upcoming.len(), 2);
        assert_eq!(stored.upcoming.first().unwrap().room_name, "Testhörsaal");

        insta::with_settings!({
            description => "a lecture derived from the calendar is returned by search",
        }, {
            insta::assert_yaml_snapshot!(results.0, {
                ".**.estimatedTotalHits" => "[estimatedTotalHits]",
                ".**.next_occurrence_at" => "[next_occurrence_at]",
                ".**.start_at" => "[start_at]",
                ".**.end_at" => "[end_at]",
            });
        });

        // Stale cleanup: once the calendar rows are gone, the next derivation
        // must delete the now-orphaned lecture document from the index.
        sqlx::query("DELETE FROM calendar")
            .execute(&pg.pool)
            .await
            .unwrap();
        crate::refresh::lectures::refresh_once(&pg.pool, &ms.client)
            .await
            .unwrap();
        let after = do_geoentry_search(
            &ms.client,
            "Quantenfeldtheorie",
            Limits::default(),
            FormattingConfig::default(),
            String::new(),
            vec![],
        )
        .await;
        assert!(
            after
                .0
                .iter()
                .find_map(ResultsSection::lectures)
                .is_none_or(|s| s.entries.is_empty()),
            "the lecture must be gone once its calendar rows are deleted"
        );
    }

    /// The `upcoming` cap is `max(next 10 occurrences, 14-day window)`: a daily
    /// tutorial is bounded by the window (~14 entries), a weekly lecture by the
    /// count (10 entries). This seeds both, derives, and asserts each arm of the
    /// `max` independently from the stored documents.
    #[tokio::test]
    #[tracing_test::traced_test]
    #[expect(
        clippy::too_many_lines,
        reason = "one test seeds Postgres + Meilisearch, derives, and asserts both arms of the cap"
    )]
    async fn test_lecture_upcoming_capped_at_max_of_top10_and_14day_window() {
        const DAY: i64 = 24 * 60 * 60;

        /// Only the fields the cap assertions need off the stored document.
        #[derive(serde::Deserialize)]
        struct StoredLecture {
            title_de: String,
            upcoming: Vec<UpcomingEvent>,
        }

        let pg = PostgresTestContainer::new().await;
        let ms = MeiliSearchTestContainer::new().await;
        let entries = ms.client.index("entries");

        let room_code = "5606.EG.011";
        let room_data = serde_json::json!({
            "name": "Testhörsaal",
            "type": "room",
            "type_common_name": "Hörsaal",
            "coords": { "lat": 48.0, "lon": 11.0, "source": "navigatum" },
        })
        .to_string();
        // `calendar.room_code` is a foreign key into `en` (which references `de`).
        sqlx::query("INSERT INTO de (key, data) VALUES ($1, $2::jsonb)")
            .bind(room_code)
            .bind(&room_data)
            .execute(&pg.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO en (key, data) VALUES ($1, $2::jsonb)")
            .bind(room_code)
            .bind(&room_data)
            .execute(&pg.pool)
            .await
            .unwrap();
        entries
            .add_documents(
                &[serde_json::json!({
                    "ms_id": "5606-EG-011",
                    "facet": "room",
                    "type": "room",
                    "room_code": room_code,
                    "name": "Testhörsaal",
                    "type_common_name": "Hörsaal",
                    "rank": 100,
                })],
                Some("ms_id"),
            )
            .await
            .unwrap()
            .wait_for_completion(&ms.client, None, Some(std::time::Duration::from_secs(30)))
            .await
            .unwrap();

        let now = Utc::now().timestamp();
        let mut id = 0;
        let mut insert = async |title: &str, start_secs: i64| {
            id += 1;
            let start = DateTime::from_timestamp(now + start_secs, 0).unwrap();
            let end = start + chrono::Duration::hours(1);
            sqlx::query(
                "INSERT INTO calendar \
                 (id, room_code, start_at, end_at, title_de, title_en, stp_type, entry_type, detailed_entry_type) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(id)
            .bind(room_code)
            .bind(start)
            .bind(end)
            .bind(title)
            .bind(title)
            .bind(Some("Vorlesung"))
            .bind("lecture")
            .bind("Vorlesung")
            .execute(&pg.pool)
            .await
            .unwrap();
        };

        // Daily tutorial: 20 future days. The 14-day window covers days 1..=14
        // (14 events), beating the top-10, so the cap is the window.
        for day in 1..=20 {
            insert("Tägliche Übung", day * DAY).await;
        }
        // Weekly lecture: 12 future weeks. The window covers only weeks 1..=2 (2
        // events), so the top-10 wins and the cap is the count.
        for week in 1..=12 {
            insert("Wöchentliche Vorlesung", week * 7 * DAY).await;
        }

        crate::refresh::lectures::refresh_once(&pg.pool, &ms.client)
            .await
            .unwrap();

        let stored = meilisearch_sdk::documents::DocumentsQuery::new(&entries)
            .with_filter("facet = \"lecture\"")
            .execute::<StoredLecture>()
            .await
            .unwrap()
            .results;
        let by_title = |title: &str| {
            stored
                .iter()
                .find(|l| l.title_de == title)
                .unwrap_or_else(|| panic!("expected a stored lecture titled {title:?}"))
        };

        let daily = by_title("Tägliche Übung");
        assert_eq!(
            daily.upcoming.len(),
            14,
            "the daily tutorial is capped by the 14-day window"
        );
        let weekly = by_title("Wöchentliche Vorlesung");
        assert_eq!(
            weekly.upcoming.len(),
            10,
            "the weekly lecture is capped by the next-10 count"
        );

        // Both are in chronological order with the room name resolved.
        for lecture in [daily, weekly] {
            assert!(
                lecture.upcoming.is_sorted_by_key(|e| e.start_at),
                "upcoming occurrences must be chronological"
            );
            assert!(
                lecture
                    .upcoming
                    .iter()
                    .all(|e| e.room_name == "Testhörsaal"),
                "every occurrence resolves its room display name"
            );
        }
    }
}
