use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;

use actix_web::web::{Data, Json};
use actix_web::{HttpResponse, post};
use serde::Deserialize;
use tracing::{error, info};
#[expect(
    unused_imports,
    reason = "has to be imported as otherwise utoipa generates incorrect code"
)]
use url::Url;

use crate::limited::hash_map::LimitedHashMap;

use super::proposed_edits::addition::Addition;
use super::proposed_edits::coordinate::Coordinate;
use super::proposed_edits::image::Image;
use super::proposed_edits::opening_hours::OpeningHoursEdit;
use super::proposed_edits::property::PropertyEdit;
use super::tokens::RecordedTokens;
use crate::external::github::{GitHub, PrCreated};

pub(crate) mod addition;
mod coordinate;
mod csv_edit;
mod description;
mod image;
mod opening_hours;
mod property;
pub(crate) mod repo_pool;
mod tmp_repo;

const COMBINED_CAP: usize = 500;

#[derive(Debug, Deserialize, Clone, utoipa::ToSchema)]
struct Edit {
    coordinate: Option<Coordinate>,
    image: Option<Image>,
    properties: Option<Vec<PropertyEdit>>,
    opening_hours: Option<OpeningHoursEdit>,
}
pub trait AppliableEdit {
    fn apply(&self, key: &str, base_dir: &Path, branch: &str) -> anyhow::Result<String>;
}

#[derive(Debug, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct EditRequest {
    /// The JWT token, that can be used to generate feedback
    #[schema(
        example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2Njk2MzczODEsImlhdCI6MTY2OTU5NDE4MSwibmJmIjoxNjY5NTk0MTkxLCJraWQiOjE1ODU0MTUyODk5MzI0MjU0Mzg2fQ.sN0WwXzsGhjOVaqWPe-Fl5x-gwZvh28MMUM-74MoNj4"
    )]
    token: String,
    /// The edits to be made to the room. The keys are the ID of the props to be edited, the values are the proposed Edits.
    #[serde(default = "LimitedHashMap::empty")]
    edits: LimitedHashMap<String, Edit>,
    /// New rooms/buildings/POIs to add. Keyed by the new entry's ID. Validated server-side.
    #[serde(default = "LimitedHashMap::empty")]
    pub(super) additions: LimitedHashMap<String, Addition>,
    /// Additional context for the edit.
    ///
    /// Will be displayed in the discription field of the PR
    #[schema(example = "I have a picture of the room, please add it to the roomfinder")]
    pub(super) additional_context: String,
    /// Whether the user has checked the privacy-checkbox.
    ///
    /// We are posting the feedback publicly on GitHub (not a EU-Company).
    /// **You MUST also include such a checkmark.**
    privacy_checked: bool,
}

pub enum ApplyError {
    // Split out from `Other` so the handler can return a structured 422 with a per-key error
    // list instead of a generic 500. Covers both addition and property-edit validation.
    Validation(Vec<ValidationFailure>),
    Other(anyhow::Error),
}

#[derive(Debug, serde::Serialize)]
pub struct ValidationFailure {
    pub key: String,
    pub error: String,
}

impl<E> From<E> for ApplyError
where
    E: Into<anyhow::Error>,
{
    fn from(e: E) -> Self {
        Self::Other(e.into())
    }
}

impl EditRequest {
    #[tracing::instrument(skip(repo_pool))]
    async fn apply_changes_and_generate_description(
        &self,
        repo_pool: &repo_pool::RepoPool,
        branch_name: &str,
        branch_is_new: bool,
    ) -> Result<description::Description, ApplyError> {
        let worktree = repo_pool
            .create_worktree(branch_name, branch_is_new)
            .await?;

        // Reject malformed additions before any writes so a bad request never produces a PR.
        if !self.additions.0.is_empty() {
            let snap = addition::validation::RepoSnapshot::load(worktree.base_dir())?;
            let mut failures = Vec::new();
            for (key, addition) in &self.additions.0 {
                if let Err(e) = addition.validate(key, &snap) {
                    failures.push(ValidationFailure {
                        key: key.clone(),
                        error: e.to_string(),
                    });
                }
            }
            if !failures.is_empty() {
                return Err(ApplyError::Validation(failures));
            }
        }

        // Reject property edits whose name looks like a pipeline-generated display
        // string before any writes, so a stale or third-party client cannot
        // launder a `{id} (...)` value into the curated names.csv (#3181).
        let mut property_failures = Vec::new();
        for (key, edit) in &self.edits.0 {
            for property in edit.properties.iter().flatten() {
                if let Err(e) = property.validate(key) {
                    property_failures.push(ValidationFailure {
                        key: key.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }
        if !property_failures.is_empty() {
            return Err(ApplyError::Validation(property_failures));
        }

        let desc = worktree.apply_and_gen_description(self, branch_name)?;
        worktree.commit(&desc.title).await?;
        worktree.push().await?;
        Ok(desc)
    }
    fn edits_for<T: AppliableEdit>(&self, extractor: fn(Edit) -> Option<T>) -> HashMap<String, T> {
        self.edits
            .0
            .clone()
            .into_iter()
            .filter_map(|(k, edit)| extractor(edit).map(|coord| (k, coord)))
            .collect()
    }

    /// True iff every payload in this request is a coordinate edit -
    /// at least one coordinate edit, no other edit kinds, no additions.
    ///
    /// Used to gate auto-merge: maintainers asked for coordinate-only PRs to merge
    /// without review because they're low-risk spot fixes, but any mix-in (image,
    /// property, `opening_hours`, or a new room/POI/event) brings the PR back into
    /// the manual-review bucket.
    pub(super) fn is_pure_coordinate_edit(&self) -> bool {
        if !self.additions.0.is_empty() {
            return false;
        }
        let mut has_coord = false;
        for edit in self.edits.0.values() {
            if edit.image.is_some()
                || edit.opening_hours.is_some()
                || edit.properties.as_ref().is_some_and(|p| !p.is_empty())
            {
                return false;
            }
            if edit.coordinate.is_some() {
                has_coord = true;
            }
        }
        has_coord
    }

    pub(super) fn extract_labels(&self) -> Vec<String> {
        let mut labels = vec!["webform".to_string()];

        if self
            .edits
            .0
            .iter()
            .any(|(_, edit)| edit.coordinate.is_some())
        {
            labels.push("coordinate".to_string());
        }
        if self.edits.0.iter().any(|(_, edit)| edit.image.is_some()) {
            labels.push("image".to_string());
        }
        if self
            .edits
            .0
            .iter()
            .any(|(_, edit)| edit.properties.as_ref().is_some_and(|p| !p.is_empty()))
        {
            labels.push("property".to_string());
        }
        if self
            .edits
            .0
            .iter()
            .any(|(_, edit)| edit.opening_hours.is_some())
        {
            labels.push("opening_hours".to_string());
        }
        if !self.additions.0.is_empty() {
            labels.push("addition".to_string());
        }
        let kinds: BTreeSet<&'static str> = self
            .additions
            .0
            .values()
            .map(Addition::kind_label)
            .collect();
        for kind in kinds {
            labels.push(format!("new-{kind}"));
        }
        labels
    }

    fn extract_subject(&self, updated_event_keys: &BTreeSet<String>) -> String {
        use itertools::Itertools as _;
        let coordinate_edits = self.edits_for(|edit| edit.coordinate);
        let image_edits = self.edits_for(|edit| edit.image);
        let property_count: usize = self
            .edits
            .0
            .values()
            .filter_map(|e| e.properties.as_ref())
            .map(Vec::len)
            .sum();

        let mut parts = Vec::new();
        match coordinate_edits.len() {
            0 => {}
            1..=5 => parts.push(format!(
                "coordinate edit for `{}`",
                coordinate_edits.keys().sorted().join("`, `")
            )),
            cs => parts.push(format!("edited {cs} coordinates")),
        }
        match image_edits.len() {
            0 => {}
            1 => parts.push(format!(
                "add image for `{}`",
                image_edits
                    .keys()
                    .next()
                    .expect("len()==1 guarantees a first key")
            )),
            2..=5 => parts.push(format!(
                "add images for `{}`",
                image_edits.keys().sorted().join("`, `")
            )),
            is => parts.push(format!("add {is} images")),
        }
        if property_count > 0 {
            let edits = if property_count == 1 { "edit" } else { "edits" };
            parts.push(format!("{property_count} property {edits}"));
        }
        let opening_hours_edits = self.edits_for(|edit| edit.opening_hours);
        match opening_hours_edits.len() {
            0 => {}
            1..=5 => parts.push(format!(
                "opening-hours edit for `{}`",
                opening_hours_edits.keys().sorted().join("`, `")
            )),
            cs => parts.push(format!("edited {cs} opening-hours schedules")),
        }

        let mut keys_by_kind: BTreeMap<&'static str, Vec<&str>> = BTreeMap::new();
        for (key, addition) in &self.additions.0 {
            // Events render by name (add/update) in a separate pass below.
            if addition.event_name().is_some() {
                continue;
            }
            keys_by_kind
                .entry(addition.kind_label())
                .or_default()
                .push(key.as_str());
        }
        for (kind, keys) in keys_by_kind {
            let plural = match kind {
                "room" => "rooms",
                "building" => "buildings",
                "poi" => "POIs",
                _ => "entries",
            };
            let singular = match kind {
                "poi" => "POI",
                other => other,
            };
            match keys.as_slice() {
                [] => {}
                [only] => parts.push(format!("add {singular} `{only}`")),
                many if many.len() <= 5 => parts.push(format!(
                    "add {plural} `{}`",
                    many.iter().sorted().join("`, `")
                )),
                many => parts.push(format!("add {} {plural}", many.len())),
            }
        }

        parts.extend(self.event_subject_parts(updated_event_keys));

        if parts.is_empty() {
            "no edits".to_string()
        } else {
            parts.join(" and ")
        }
    }

    /// Subject fragments for event additions, by name, distinguishing "add" from "update".
    fn event_subject_parts(&self, updated_event_keys: &BTreeSet<String>) -> Vec<String> {
        let mut added = Vec::new();
        let mut updated = Vec::new();
        for (key, addition) in &self.additions.0 {
            if let Some(name) = addition.event_name() {
                if updated_event_keys.contains(key) {
                    updated.push(name);
                } else {
                    added.push(name);
                }
            }
        }
        let mut parts = Vec::new();
        for (verb, mut names) in [("add", added), ("update", updated)] {
            names.sort_unstable();
            match names.as_slice() {
                [] => {}
                [only] => parts.push(format!("{verb} event \"{only}\"")),
                many if many.len() <= 5 => {
                    parts.push(format!("{verb} events \"{}\"", many.join("\", \"")));
                }
                many => parts.push(format!("{verb} {} events", many.len())),
            }
        }
        parts
    }
}

/// Enable squash auto-merge on a freshly-opened pure-coordinate PR.
/// Best-effort: failures here only mean the PR will sit waiting for a manual merge.
#[tracing::instrument(skip(created))]
async fn enable_auto_merge_for_pure_coord(created: &PrCreated) {
    if let Err(e) = GitHub::default()
        .enable_auto_merge_squash(&created.node_id)
        .await
    {
        error!(
            error = ?e,
            pr_number = created.number,
            "Failed to enable auto-merge on pure-coordinate PR"
        );
    }
}

/// Take an in-progress batch PR off auto-merge after a non-coordinate edit lands in it.
/// Logged at `warn!` on failure: an undiscovered failure here could leave a mixed batch
/// PR auto-merging once CI passes, which is the exact scenario this gate is supposed to prevent.
/// (The most common cause is "auto-merge was never enabled," which is a benign no-op but still
/// worth surfacing so operators can tune the gate if the warnings are dominated by that case.)
#[tracing::instrument]
async fn disable_auto_merge_on_mixed_batch(pr_number: u64) {
    match GitHub::default().pr_node_id(pr_number).await {
        Ok(node_id) => {
            if let Err(e) = GitHub::default().disable_auto_merge(&node_id).await {
                tracing::warn!(
                    error = ?e,
                    %pr_number,
                    "disable_auto_merge failed; mixed batch PR may still auto-merge if it was previously enabled"
                );
            }
        }
        Err(e) => {
            error!(error = ?e, %pr_number, "Failed to fetch node_id for batch PR");
        }
    }
}

/// Post Edit-Requests
///
/// ***Do not abuse this endpoint.***
///
/// This posts the actual feedback to GitHub and returns the github link.
/// This API will create pull-requests instead of issues => only a subset of feedback is allowed.
/// For this Endpoint to work, you need to generate a token via the [`/api/feedback/get_token`](#tag/feedback/operation/get_token) endpoint.
///
/// # Note:
///
/// Tokens are only used if we return a 201 Created response. Otherwise, they are still valid
#[utoipa::path(
    tags=["feedback"],
    responses(
        (status = 201, description= "The edit request feedback has been **successfully posted to GitHub**. We return the link to the GitHub issue.", body= Url, content_type="text/plain", example="https://github.com/TUM-Dev/navigatum/issues/9"),
        (status = 400, description= "**Bad Request.** Not all fields in the body are present as defined above"),
        (status = 403, description= r"**Forbidden.** Causes are (delivered via the body):

- `Invalid token`: You have not supplied a token generated via the `gen_token`-Endpoint.
- `Token not old enough, please wait`: Tokens are only valid after 10s.
- `Token expired`: Tokens are only valid for 12h.
- `Token already used`: Tokens are non reusable/refreshable single-use items."),
        (status = 422, description= "**Unprocessable Entity.** Subject or body missing or too short."),
        (status = 451, description= "**Unavailable for legal reasons.** Using this endpoint without accepting the privacy policy is not allowed. For us to post to GitHub, this has to be true"),
        (status = 500, description= "**Internal Server Error.** We have a problem communicating with GitHubs servers. Please try again later."),
        (status = 503, description= "Service unavailable. We have not configured a GitHub Access Token. This could be because we are experiencing technical difficulties or intentional. Please try again later."),
    )
)]
#[post("/api/feedback/propose_edits")]
pub async fn propose_edits(
    recorded_tokens: Data<RecordedTokens>,
    repo_pool: Data<Arc<repo_pool::RepoPool>>,
    req_data: Json<EditRequest>,
) -> HttpResponse {
    // auth
    if let Some(e) = recorded_tokens.validate(&req_data.token).await {
        return e;
    }

    // validate request
    if !req_data.privacy_checked {
        return HttpResponse::UnavailableForLegalReasons()
            .content_type("text/plain")
            .body("Using this endpoint without accepting the privacy policy is not allowed");
    }
    if req_data.edits.0.is_empty() && req_data.additions.0.is_empty() {
        return HttpResponse::UnprocessableEntity()
            .content_type("text/plain")
            .body("Not enough edits or additions provided");
    }
    let combined_count = req_data.edits.0.len() + req_data.additions.0.len();
    if combined_count > COMBINED_CAP {
        return HttpResponse::InsufficientStorage()
            .content_type("text/plain")
            .body("Too many edits + additions provided");
    }

    let branch_name = format!("usergenerated/request-{}", rand::random::<u16>());

    // Serialize the full fetch→edit→push→PR-update cycle to avoid
    // non-fast-forward push failures on the shared batch branch.
    let _branch_guard = repo_pool.branch_mutex.lock().await;

    // Try to find an open batch PR and use it
    let batch_pr = super::batch_processor::find_open_batch_pr()
        .await
        .ok()
        .flatten();

    let (branch_to_use, pr_number_opt) = match batch_pr {
        Some((pr_number, batch_branch)) => {
            info!(%pr_number, "Adding edit to existing batch PR");
            (batch_branch, Some(pr_number))
        }
        None => (branch_name, None),
    };
    let branch_is_new = pr_number_opt.is_none();

    match req_data
        .apply_changes_and_generate_description(&repo_pool, &branch_to_use, branch_is_new)
        .await
    {
        Ok(desc) => {
            if let Some(pr_number) = pr_number_opt {
                // Update metadata for batch PR (including appending description)
                if let Err(e) = super::batch_processor::update_batch_pr_metadata(
                    pr_number,
                    &req_data,
                    &desc.body,
                )
                .await
                {
                    error!(error = ?e, "Failed to update batch PR metadata");
                }

                // A non-coordinate edit landing on an in-progress batch invalidates the
                // pure-coordinate auto-merge invariant.
                if !req_data.is_pure_coordinate_edit() {
                    disable_auto_merge_on_mixed_batch(pr_number).await;
                }

                let pr_url = format!("https://github.com/TUM-Dev/NavigaTUM/pull/{pr_number}");
                HttpResponse::Created()
                    .content_type("text/plain")
                    .body(pr_url)
            } else {
                // Create new batch PR with batch-in-progress label
                let mut labels = req_data.extract_labels();
                labels.push(super::batch_processor::BATCH_LABEL.to_string());

                // Use extract_subject for first PR to provide helpful context
                let subject = req_data.extract_subject(&desc.updated_event_keys);
                let title = format!("chore(data): {subject}");

                match GitHub::default()
                    .open_pr(
                        branch_to_use,
                        &title,
                        &format!("<sub>Batched edits</sub>\n\n{}", desc.body),
                        labels,
                    )
                    .await
                {
                    Ok(created) => {
                        if req_data.is_pure_coordinate_edit() {
                            enable_auto_merge_for_pure_coord(&created).await;
                        }
                        HttpResponse::Created()
                            .content_type("text/plain")
                            .body(created.html_url)
                    }
                    Err(resp) => resp,
                }
            }
        }
        Err(ApplyError::Validation(failures)) => {
            info!(?failures, "edit validation failed");
            HttpResponse::UnprocessableEntity()
                .content_type("application/json")
                .body(serde_json::to_string(&failures).unwrap_or_else(|_| "[]".to_string()))
        }
        Err(ApplyError::Other(error)) => {
            error!(?error, "could not apply changes");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Could apply changes, please try again later")
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::panic_in_result_fn,
        reason = "tests assert via panic/unwrap"
    )]
    use super::*;

    fn req_with_additions(json: serde_json::Value) -> EditRequest {
        serde_json::from_value(json).unwrap()
    }

    fn coords() -> serde_json::Value {
        serde_json::json!({"lat": 48.262, "lon": 11.668})
    }

    fn event_addition() -> serde_json::Value {
        serde_json::json!({
            "kind": "event",
            "image": { "content": "AAAA", "metadata": { "author": "Studi", "license": { "text": "CC-BY" } } },
            "name": "GARNIX Festival",
            "description": "Open-air student festival.",
            "starts_at": "2026-06-10T16:00:00+02:00",
            "ends_at": "2026-06-12T23:00:00+02:00",
            "coords": coords(),
            "organising_org_id": 51897
        })
    }

    #[test]
    fn extract_subject_and_labels_for_event() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "additions": { "event_9d02ddd940c43f87": event_addition() },
            "additional_context": "",
            "privacy_checked": true
        }));
        assert_eq!(
            req.extract_subject(&BTreeSet::new()),
            "add event \"GARNIX Festival\""
        );
        let labels = req.extract_labels();
        assert!(labels.contains(&"addition".to_string()));
        assert!(labels.contains(&"new-event".to_string()));
    }

    #[test]
    fn extract_subject_for_event_that_replaced_an_existing_one() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "additions": { "event_9d02ddd940c43f87": event_addition() },
            "additional_context": "",
            "privacy_checked": true
        }));
        let updated = BTreeSet::from(["event_9d02ddd940c43f87".to_string()]);
        assert_eq!(
            req.extract_subject(&updated),
            "update event \"GARNIX Festival\""
        );
    }

    #[test]
    fn extract_subject_pure_addition_room() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "additions": {
                "5117.EG.103": {
                    "kind": "room",
                    "parent_building_id": "5117",
                    "alt_name": "Testraum",
                    "arch_name": "EG103@5117",
                    "usage_id": 12,
                    "coords": coords()
                }
            },
            "additional_context": "",
            "privacy_checked": true
        }));
        assert_eq!(
            req.extract_subject(&BTreeSet::new()),
            "add room `5117.EG.103`"
        );
    }

    #[test]
    fn extract_subject_multiple_rooms() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "additions": {
                "5117.EG.103": {"kind": "room", "parent_building_id": "5117", "alt_name": "A", "arch_name": "EG103@5117", "usage_id": 12, "coords": coords()},
                "5117.EG.104": {"kind": "room", "parent_building_id": "5117", "alt_name": "B", "arch_name": "EG104@5117", "usage_id": 12, "coords": coords()}
            },
            "additional_context": "",
            "privacy_checked": true
        }));
        let subj = req.extract_subject(&BTreeSet::new());
        assert!(subj.starts_with("add rooms `"));
        assert!(subj.contains("5117.EG.103"));
        assert!(subj.contains("5117.EG.104"));
    }

    #[test]
    fn extract_subject_many_pois() {
        let mut additions = serde_json::Map::new();
        for i in 0..10 {
            additions.insert(
                format!("validierungsautomat-{i:02}"),
                serde_json::json!({"kind": "poi", "parent": "0501", "name": format!("V{i}"), "usage_name": "x", "coords": coords()}),
            );
        }
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "additions": additions,
            "additional_context": "",
            "privacy_checked": true
        }));
        assert_eq!(req.extract_subject(&BTreeSet::new()), "add 10 POIs");
    }

    #[test]
    fn extract_labels_includes_addition_kinds() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "additions": {
                "5117.EG.103": {"kind": "room", "parent_building_id": "5117", "alt_name": "A", "arch_name": "EG103@5117", "usage_id": 12, "coords": coords()},
                "validierungsautomat-99": {"kind": "poi", "parent": "0501", "name": "V", "usage_name": "x", "coords": coords()}
            },
            "additional_context": "",
            "privacy_checked": true
        }));
        let labels = req.extract_labels();
        assert!(labels.contains(&"webform".to_string()));
        assert!(labels.contains(&"addition".to_string()));
        assert!(labels.contains(&"new-room".to_string()));
        assert!(labels.contains(&"new-poi".to_string()));
        assert!(!labels.contains(&"new-building".to_string()));
    }

    #[test]
    fn extract_subject_and_labels_for_opening_hours() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "edits": {
                "5304.EG.001": {
                    "opening_hours": {
                        "opening_hours": "Mo-Fr 08:00-20:00",
                        "source_url": "https://www.ub.tum.de/oeffnungszeiten"
                    }
                }
            },
            "additional_context": "",
            "privacy_checked": true
        }));
        assert_eq!(
            req.extract_subject(&BTreeSet::new()),
            "opening-hours edit for `5304.EG.001`"
        );
        assert!(req.extract_labels().contains(&"opening_hours".to_string()));
    }

    #[test]
    fn pure_coordinate_single_edit() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "edits": {
                "0101": {"coordinate": {"lat": 1.0, "lon": 1.0}}
            },
            "additional_context": "",
            "privacy_checked": true
        }));
        assert!(req.is_pure_coordinate_edit());
    }

    #[test]
    fn pure_coordinate_multiple_keys() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "edits": {
                "0101": {"coordinate": {"lat": 1.0, "lon": 1.0}},
                "0102": {"coordinate": {"lat": 2.0, "lon": 2.0}}
            },
            "additional_context": "",
            "privacy_checked": true
        }));
        assert!(req.is_pure_coordinate_edit());
    }

    #[test]
    fn not_pure_coordinate_with_image() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "edits": {
                "0101": {"coordinate": {"lat": 1.0, "lon": 1.0}},
                "0102": {
                    "image": {
                        "content": "AAAA",
                        "metadata": { "author": "Studi", "license": { "text": "CC-BY" } }
                    }
                }
            },
            "additional_context": "",
            "privacy_checked": true
        }));
        assert!(!req.is_pure_coordinate_edit());
    }

    #[test]
    fn not_pure_coordinate_with_addition() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "edits": {
                "0101": {"coordinate": {"lat": 1.0, "lon": 1.0}}
            },
            "additions": {
                "5117.EG.103": {
                    "kind": "room",
                    "parent_building_id": "5117",
                    "alt_name": "A",
                    "arch_name": "EG103@5117",
                    "usage_id": 12,
                    "coords": coords()
                }
            },
            "additional_context": "",
            "privacy_checked": true
        }));
        assert!(!req.is_pure_coordinate_edit());
    }

    #[test]
    fn not_pure_coordinate_with_opening_hours() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "edits": {
                "0101": {"coordinate": {"lat": 1.0, "lon": 1.0}},
                "5304.EG.001": {
                    "opening_hours": {
                        "opening_hours": "Mo-Fr 08:00-20:00",
                        "source_url": "https://www.ub.tum.de/oeffnungszeiten"
                    }
                }
            },
            "additional_context": "",
            "privacy_checked": true
        }));
        assert!(!req.is_pure_coordinate_edit());
    }

    #[test]
    fn not_pure_coordinate_with_property() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "edits": {
                "0101": {
                    "coordinate": {"lat": 1.0, "lon": 1.0},
                    "properties": [{
                        "type": "name",
                        "name": "Test Room",
                        "short_name": null
                    }]
                }
            },
            "additional_context": "",
            "privacy_checked": true
        }));
        assert!(!req.is_pure_coordinate_edit());
    }

    #[test]
    fn not_pure_coordinate_without_any_coordinate() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "edits": {
                "5304.EG.001": {
                    "opening_hours": {
                        "opening_hours": "Mo-Fr 08:00-20:00",
                        "source_url": "https://www.ub.tum.de/oeffnungszeiten"
                    }
                }
            },
            "additional_context": "",
            "privacy_checked": true
        }));
        assert!(!req.is_pure_coordinate_edit());
    }

    #[test]
    fn pure_coordinate_with_empty_properties_vec() {
        // properties: Some(vec![]) shouldn't disqualify - only non-empty does.
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "edits": {
                "0101": {
                    "coordinate": {"lat": 1.0, "lon": 1.0},
                    "properties": []
                }
            },
            "additional_context": "",
            "privacy_checked": true
        }));
        assert!(req.is_pure_coordinate_edit());
    }

    #[test]
    fn extract_subject_combines_edits_and_additions() {
        let req = req_with_additions(serde_json::json!({
            "token": "x",
            "edits": {
                "0101": {"coordinate": {"lat": 1.0, "lon": 1.0}}
            },
            "additions": {
                "5117.EG.103": {"kind": "room", "parent_building_id": "5117", "alt_name": "A", "arch_name": "EG103@5117", "usage_id": 12, "coords": coords()}
            },
            "additional_context": "",
            "privacy_checked": true
        }));
        let subj = req.extract_subject(&BTreeSet::new());
        assert!(subj.contains("coordinate edit for `0101`"));
        assert!(subj.contains("add room `5117.EG.103`"));
        assert!(subj.contains(" and "));
    }
}
