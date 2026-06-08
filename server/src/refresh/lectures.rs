//! Derives the `lecture` search facet from the `calendar` table.
//!
//! Lectures live in the same Meilisearch `entries` index as the geo-entries, as
//! a fifth facet. Every tick re-derives one document per distinct lecture
//! identity from the upcoming `calendar` rows, bulk-upserts them, and deletes
//! lecture documents whose identity no longer appears (renames, semester-end
//! cleanup, `stp_type` changes). The task runs once at startup and then on a
//! fixed cadence so the index is non-empty before the first user query.

use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;

use chrono::{DateTime, Utc};
use meilisearch_sdk::client::Client;
use meilisearch_sdk::documents::DocumentsQuery;
use meilisearch_sdk::tasks::Task;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::Json;
use tokio::time::sleep;
use tracing::{debug, error, info};
use xxhash_rust::xxh3::xxh3_64;

use crate::external::meilisearch::{
    ENTRIES_INDEX, FACET_FIELD, LECTURE_FACET, ROOM_FACET, UpcomingEvent,
};

/// How often the lecture facet is re-derived after the initial startup run.
const REFRESH_INTERVAL: Duration = Duration::from_mins(5);
const TIMEOUT: Option<Duration> = Some(Duration::from_mins(1));
const POLLING_RATE: Option<Duration> = Some(Duration::from_millis(250));
/// Page size for streaming geo room documents back out of the index. The index
/// holds a few thousand rooms, so a handful of pages covers it.
const PAGE_SIZE: usize = 1_000;
/// Fallback `type_common_name` when a lecture group has no `stp_type`.
const DEFAULT_TYPE_COMMON_NAME: &str = "Lehrveranstaltung";

/// Re-derive the lecture facet immediately, then every [`REFRESH_INTERVAL`].
#[tracing::instrument(skip(pool, client))]
pub async fn refresh_lectures(pool: PgPool, client: Client) {
    loop {
        if let Err(e) = refresh_once(&pool, &client).await {
            error!(error = ?e, "could not refresh the lecture search facet");
        }
        sleep(REFRESH_INTERVAL).await;
    }
}

#[tracing::instrument(skip(pool, client))]
pub(crate) async fn refresh_once(pool: &PgPool, client: &Client) -> anyhow::Result<()> {
    let groups = aggregate_lectures(pool).await?;
    let room_context = fetch_room_context(client).await?;

    let documents: Vec<LectureDocument> = groups
        .iter()
        .map(|group| LectureDocument::from_group(group, &room_context))
        .collect();
    let produced_ids: HashSet<String> = documents.iter().map(|d| d.ms_id.clone()).collect();

    let entries = client.index(ENTRIES_INDEX);
    if !documents.is_empty() {
        let res = entries
            .add_documents(&documents, Some("ms_id"))
            .await?
            .wait_for_completion(client, POLLING_RATE, TIMEOUT)
            .await?;
        if let Task::Failed { content } = res {
            anyhow::bail!("Failed to upsert lecture documents into Meilisearch: {content:?}");
        }
    }

    let stale = stale_lecture_ids(client, &produced_ids).await?;
    if !stale.is_empty() {
        let res = entries
            .delete_documents(&stale)
            .await?
            .wait_for_completion(client, POLLING_RATE, TIMEOUT)
            .await?;
        if let Task::Failed { content } = res {
            anyhow::bail!("Failed to delete stale lecture documents from Meilisearch: {content:?}");
        }
    }

    info!(
        upserted = documents.len(),
        deleted = stale.len(),
        "refreshed the lecture search facet"
    );
    Ok(())
}

/// One upcoming occurrence as aggregated from `calendar`.
///
/// The room is still only a code here; its display name is resolved against the
/// geo room documents when the stored [`UpcomingEvent`] is built.
#[derive(Deserialize)]
struct UpcomingEventRaw {
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    room_code: String,
}

/// One distinct lecture identity, aggregated from upcoming `calendar` rows.
struct LectureGroup {
    /// Lowercased German title - part of the identity key.
    key_title_de: String,
    /// Lowercased English title - part of the identity key.
    key_title_en: String,
    /// `stp_type` folded to `''` when absent - part of the identity key.
    key_stp_type: String,
    /// Display German title, taken from the next upcoming occurrence.
    title_de: String,
    /// Display English title, taken from the next upcoming occurrence.
    title_en: String,
    /// The raw `stp_type`, if any, surfaced as the human type label.
    stp_type: Option<String>,
    /// Earliest `start_at` among rows that have not yet ended.
    next_occurrence_at: DateTime<Utc>,
    /// Distinct rooms hosting upcoming occurrences of this lecture.
    room_codes: Vec<String>,
    /// The next occurrences in chronological order, capped at whichever covers
    /// more events: the next 10 or those within a 14-day window (see
    /// [`aggregate_lectures`]).
    upcoming: Json<Vec<UpcomingEventRaw>>,
}

#[tracing::instrument(skip(pool))]
async fn aggregate_lectures(pool: &PgPool) -> anyhow::Result<Vec<LectureGroup>> {
    // Group identity is `(LOWER(title_de), LOWER(title_en), COALESCE(stp_type, ''))`:
    // both languages so a rename in one locale cannot collide identities, and a
    // lowercase fold so scrape-time capitalisation drift does not fork a group.
    // Display titles and `stp_type` are read off the earliest upcoming row so the
    // surfaced text matches what the user is about to attend.
    //
    // `upcoming` materialises the per-occurrence rows the expandable lecture row
    // renders. It is capped at `max(next 10 occurrences, 14-day window)` -
    // whichever covers more events wins, so a weekly lecture keeps ~10 entries
    // and a daily tutorial ~14, with a predictable bound on document size. Both
    // sets are prefixes of the chronological order, so their union is just the
    // longer prefix: `rn <= 10 OR start_at <= NOW() + 14 days`. `rn = 1` always
    // passes the filter, so the aggregate is never `NULL`.
    let groups = sqlx::query_as!(
        LectureGroup,
        r#"
        WITH upcoming AS (
            SELECT
                LOWER(title_de)        AS key_title_de,
                LOWER(title_en)        AS key_title_en,
                COALESCE(stp_type, '') AS key_stp_type,
                title_de,
                title_en,
                stp_type,
                start_at,
                end_at,
                room_code,
                ROW_NUMBER() OVER (
                    PARTITION BY LOWER(title_de), LOWER(title_en), COALESCE(stp_type, '')
                    ORDER BY start_at, room_code
                ) AS rn
            FROM calendar
            WHERE end_at >= NOW()
        )
        SELECT
            key_title_de                                  AS "key_title_de!",
            key_title_en                                  AS "key_title_en!",
            key_stp_type                                  AS "key_stp_type!",
            (ARRAY_AGG(title_de ORDER BY start_at))[1]    AS "title_de!",
            (ARRAY_AGG(title_en ORDER BY start_at))[1]    AS "title_en!",
            (ARRAY_AGG(stp_type ORDER BY start_at))[1]    AS "stp_type",
            MIN(start_at)                                 AS "next_occurrence_at!",
            ARRAY_AGG(DISTINCT room_code)                 AS "room_codes!",
            JSONB_AGG(
                JSONB_BUILD_OBJECT('start_at', start_at, 'end_at', end_at, 'room_code', room_code)
                ORDER BY rn
            ) FILTER (
                WHERE rn <= 10 OR start_at <= NOW() + INTERVAL '14 days'
            )                                             AS "upcoming!: Json<Vec<UpcomingEventRaw>>"
        FROM upcoming
        GROUP BY key_title_de, key_title_en, key_stp_type
        "#,
    )
    .fetch_all(pool)
    .await?;
    debug!(cnt = groups.len(), "aggregated upcoming lectures");
    Ok(groups)
}

/// Context of a single room, harvested from its geo document: the display name
/// surfaced on each upcoming occurrence, plus the parent hierarchy the lecture
/// inherits for search.
#[derive(Default)]
struct RoomContext {
    name: String,
    building_names: Vec<String>,
    keywords: Vec<String>,
}

#[derive(Deserialize)]
struct RoomContextDoc {
    room_code: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    parent_building_names: Vec<String>,
    #[serde(default)]
    parent_keywords: Vec<String>,
}

/// Build a `room_code -> context` map from the geo room documents already in the
/// index. This keeps a single source of truth rather than re-deriving from
/// Postgres, so lecture rows inherit exactly the building names and keywords
/// their rooms search by (boosting e.g. "Mathe MW") and reuse the same display
/// name the room itself surfaces.
#[tracing::instrument(skip(client))]
async fn fetch_room_context(client: &Client) -> anyhow::Result<HashMap<String, RoomContext>> {
    let entries = client.index(ENTRIES_INDEX);
    let facet_filter = format!("{FACET_FIELD} = \"{ROOM_FACET}\"");
    let mut rooms = HashMap::new();
    let mut offset = 0;
    loop {
        let page = DocumentsQuery::new(&entries)
            .with_filter(&facet_filter)
            .with_fields([
                "room_code",
                "name",
                "parent_building_names",
                "parent_keywords",
            ])
            .with_limit(PAGE_SIZE)
            .with_offset(offset)
            .execute::<RoomContextDoc>()
            .await?;
        let returned = page.results.len();
        for doc in page.results {
            rooms.insert(
                doc.room_code,
                RoomContext {
                    name: doc.name,
                    building_names: doc.parent_building_names,
                    keywords: doc.parent_keywords,
                },
            );
        }
        offset += returned;
        if returned < PAGE_SIZE || offset >= page.total as usize {
            break;
        }
    }
    debug!(
        cnt = rooms.len(),
        "indexed room context for lecture derivation"
    );
    Ok(rooms)
}

/// The set of lecture `ms_id`s currently in the index that were *not* just
/// produced - i.e. the documents to delete this tick.
#[tracing::instrument(skip(client, produced_ids))]
async fn stale_lecture_ids(
    client: &Client,
    produced_ids: &HashSet<String>,
) -> anyhow::Result<Vec<String>> {
    #[derive(Deserialize)]
    struct LectureId {
        ms_id: String,
    }

    let entries = client.index(ENTRIES_INDEX);
    let facet_filter = format!("{FACET_FIELD} = \"{LECTURE_FACET}\"");
    let mut stale = Vec::new();
    let mut offset = 0;
    loop {
        let page = DocumentsQuery::new(&entries)
            .with_filter(&facet_filter)
            .with_fields(["ms_id"])
            .with_limit(PAGE_SIZE)
            .with_offset(offset)
            .execute::<LectureId>()
            .await?;
        let returned = page.results.len();
        for doc in page.results {
            if !produced_ids.contains(&doc.ms_id) {
                stale.push(doc.ms_id);
            }
        }
        offset += returned;
        if returned < PAGE_SIZE || offset >= page.total as usize {
            break;
        }
    }
    Ok(stale)
}

/// A lecture document as stored in the `entries` index.
///
/// Field names and semantics mirror the geo documents emitted by the data
/// pipeline (`name`, `rank`, `parent_*`) so the shared index settings apply
/// uniformly; the lecture-specific fields (`title_*`, `next_occurrence_at`) are
/// additive.
#[derive(Serialize)]
struct LectureDocument {
    ms_id: String,
    facet: &'static str,
    r#type: &'static str,
    type_common_name: String,
    title_de: String,
    title_en: String,
    /// Mirrors `title_de` for the monolingual `name` field geo documents use.
    name: String,
    /// Always `0` so lectures lose `rank:desc` tie-breaks against geo-entries.
    rank: i32,
    parent_building_names: Vec<String>,
    parent_keywords: Vec<String>,
    next_occurrence_at: DateTime<Utc>,
    upcoming: Vec<UpcomingEvent>,
}

impl LectureDocument {
    fn from_group(group: &LectureGroup, room_context: &HashMap<String, RoomContext>) -> Self {
        let (parent_building_names, parent_keywords) = group.parent_context(room_context);
        Self {
            ms_id: group.ms_id(),
            facet: LECTURE_FACET,
            r#type: LECTURE_FACET,
            type_common_name: group
                .stp_type
                .clone()
                .unwrap_or_else(|| DEFAULT_TYPE_COMMON_NAME.to_string()),
            title_de: group.title_de.clone(),
            title_en: group.title_en.clone(),
            name: group.title_de.clone(),
            rank: 0,
            parent_building_names,
            parent_keywords,
            next_occurrence_at: group.next_occurrence_at,
            upcoming: group.upcoming(room_context),
        }
    }
}

impl LectureGroup {
    /// Stable, identity-preserving document id derived from the group key.
    ///
    /// An `xxh3` hash of the key keeps the same lecture mapped to the same
    /// `ms_id` across ticks, which is what lets the upsert update in place and
    /// the stale-cleanup target only real removals. The id is not security
    /// sensitive, so a fast non-cryptographic hash is the right tool.
    fn ms_id(&self) -> String {
        // Unit-separator delimiters keep the three components unambiguous so two
        // different keys cannot hash to the same digest by concatenation.
        let key = format!(
            "{}\x1f{}\x1f{}",
            self.key_title_de, self.key_title_en, self.key_stp_type
        );
        // xxh3 is a 64-bit digest, so this is always 16 hex chars - ample to
        // keep lecture identities collision-free.
        let hash = xxh3_64(key.as_bytes());
        format!("lecture_{hash:016x}")
    }

    /// Union of the parent building names and keywords of every room hosting an
    /// upcoming occurrence, de-duplicated while preserving first-seen order.
    fn parent_context(
        &self,
        room_context: &HashMap<String, RoomContext>,
    ) -> (Vec<String>, Vec<String>) {
        let mut building_names = Vec::new();
        let mut keywords = Vec::new();
        let mut seen_names = HashSet::new();
        let mut seen_keywords = HashSet::new();
        for room_code in &self.room_codes {
            let Some(context) = room_context.get(room_code) else {
                continue;
            };
            for name in &context.building_names {
                if seen_names.insert(name.clone()) {
                    building_names.push(name.clone());
                }
            }
            for keyword in &context.keywords {
                if seen_keywords.insert(keyword.clone()) {
                    keywords.push(keyword.clone());
                }
            }
        }
        (building_names, keywords)
    }

    /// The capped, chronologically ordered occurrences with their rooms resolved
    /// to display names. A room missing from the index (e.g. an online-only
    /// slot) falls back to its code so the entry stays clickable.
    fn upcoming(&self, room_context: &HashMap<String, RoomContext>) -> Vec<UpcomingEvent> {
        self.upcoming
            .0
            .iter()
            .map(|event| {
                let room_name = room_context
                    .get(&event.room_code)
                    .map(|context| context.name.clone())
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| event.room_code.clone());
                UpcomingEvent {
                    start_at: event.start_at,
                    end_at: event.end_at,
                    room_code: event.room_code.clone(),
                    room_name,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "tests assert via panic/unwrap"
    )]
    use super::*;

    /// A group whose identity key is derived from the (case-folded) titles and
    /// `stp_type`, exactly as [`aggregate_lectures`] produces it. The display
    /// fields and `next_occurrence_at` are irrelevant to the id.
    fn group(title_de: &str, title_en: &str, stp_type: Option<&str>) -> LectureGroup {
        LectureGroup {
            key_title_de: title_de.to_lowercase(),
            key_title_en: title_en.to_lowercase(),
            key_stp_type: stp_type.unwrap_or_default().to_string(),
            title_de: title_de.to_string(),
            title_en: title_en.to_string(),
            stp_type: stp_type.map(str::to_string),
            next_occurrence_at: DateTime::from_timestamp(0, 0).unwrap(),
            room_codes: vec![],
            upcoming: Json(vec![]),
        }
    }

    #[test]
    fn ms_id_is_prefixed_stable_and_identity_scoped() {
        let a = group("Analysis 1", "Calculus 1", Some("Vorlesung"));

        let id = a.ms_id();
        assert!(id.starts_with("lecture_"), "got {id}");
        assert_eq!(id.len(), "lecture_".len() + 16);

        // Case drift in the display titles must not fork the identity.
        assert_eq!(
            a.ms_id(),
            group("ANALYSIS 1", "CALCULUS 1", Some("Vorlesung")).ms_id()
        );

        // Each of the three key components changes the id.
        assert_ne!(
            a.ms_id(),
            group("Analysis 2", "Calculus 1", Some("Vorlesung")).ms_id()
        );
        assert_ne!(
            a.ms_id(),
            group("Analysis 1", "Calculus 2", Some("Vorlesung")).ms_id()
        );
        assert_ne!(
            a.ms_id(),
            group("Analysis 1", "Calculus 1", Some("Übung")).ms_id()
        );

        // An absent `stp_type` folds to "" and must not collide with a real one.
        assert_ne!(
            group("X", "Y", None).ms_id(),
            group("X", "Y", Some("Vorlesung")).ms_id()
        );
    }

    #[test]
    fn ms_id_delimiter_prevents_concatenation_collisions() {
        // Without the unit-separator delimiter, ("ab", "x") and ("a", "bx")
        // would hash the same byte stream. The delimiter keeps them distinct.
        assert_ne!(
            group("ab", "x", None).ms_id(),
            group("a", "bx", None).ms_id()
        );
    }
}
