use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

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
use super::proposed_edits::property::PropertyEdit;
use super::proposed_edits::tmp_repo::TempRepo;
use super::tokens::RecordedTokens;
use crate::external::github::GitHub;

pub(crate) mod addition;
mod coordinate;
mod description;
mod image;
mod property;
mod tmp_repo;

const COMBINED_CAP: usize = 500;

#[derive(Debug, Deserialize, Clone, utoipa::ToSchema)]
struct Edit {
    coordinate: Option<Coordinate>,
    image: Option<Image>,
    properties: Option<Vec<PropertyEdit>>,
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
    // list instead of a generic 500.
    AdditionValidation(Vec<AdditionValidationFailure>),
    Other(anyhow::Error),
}

#[derive(Debug, serde::Serialize)]
pub struct AdditionValidationFailure {
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
    #[tracing::instrument]
    async fn apply_changes_and_generate_description(
        &self,
        branch_name: &str,
        branch_is_new: bool,
    ) -> Result<String, ApplyError> {
        let Some(pat) = GitHub::github_token() else {
            return Err(ApplyError::Other(anyhow::anyhow!(
                "Failed to get GitHub token"
            )));
        };
        let url = format!("https://{pat}@github.com/TUM-Dev/NavigaTUM");
        let repo = if branch_is_new {
            TempRepo::clone_and_checkout_new_branch(&url, branch_name).await?
        } else {
            TempRepo::clone_and_checkout_existing_branch(&url, branch_name).await?
        };

        // Reject malformed additions before any writes so a bad request never produces a PR.
        if !self.additions.0.is_empty() {
            let snap = addition::validation::RepoSnapshot::load(repo.base_dir())?;
            let mut failures = Vec::new();
            for (key, addition) in &self.additions.0 {
                if let Err(e) = addition.validate(key, &snap) {
                    failures.push(AdditionValidationFailure {
                        key: key.clone(),
                        error: e.to_string(),
                    });
                }
            }
            if !failures.is_empty() {
                return Err(ApplyError::AdditionValidation(failures));
            }
        }

        let desc = repo.apply_and_gen_description(self, branch_name)?;
        repo.commit(&desc.title).await?;
        repo.push().await?;
        Ok(desc.body)
    }
    fn edits_for<T: AppliableEdit>(&self, extractor: fn(Edit) -> Option<T>) -> HashMap<String, T> {
        self.edits
            .0
            .clone()
            .into_iter()
            .filter_map(|(k, edit)| extractor(edit).map(|coord| (k, coord)))
            .collect()
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

    fn extract_subject(&self) -> String {
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

        let mut keys_by_kind: BTreeMap<&'static str, Vec<&str>> = BTreeMap::new();
        for (key, addition) in &self.additions.0 {
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
            match keys.len() {
                0 => {}
                1 => parts.push(format!("add {singular} `{}`", keys[0])),
                2..=5 => parts.push(format!(
                    "add {plural} `{}`",
                    keys.iter().sorted().join("`, `")
                )),
                n => parts.push(format!("add {n} {plural}")),
            }
        }

        if parts.is_empty() {
            "no edits".to_string()
        } else {
            parts.join(" and ")
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
        .apply_changes_and_generate_description(&branch_to_use, branch_is_new)
        .await
    {
        Ok(description) => {
            if let Some(pr_number) = pr_number_opt {
                // Update metadata for batch PR (including appending description)
                if let Err(e) = super::batch_processor::update_batch_pr_metadata(
                    pr_number,
                    &req_data,
                    &description,
                )
                .await
                {
                    error!(error = ?e, "Failed to update batch PR metadata");
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
                let subject = req_data.extract_subject();
                let title = format!("chore(data): {subject}");

                GitHub::default()
                    .open_pr(
                        branch_to_use,
                        &title,
                        &format!("## Batched Edits\n\n### Edit #1\n{description}"),
                        labels,
                    )
                    .await
            }
        }
        Err(ApplyError::AdditionValidation(failures)) => {
            info!(?failures, "addition validation failed");
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
#[allow(clippy::unwrap_used, clippy::panic, clippy::panic_in_result_fn)]
mod tests {
    use super::*;

    fn req_with_additions(json: serde_json::Value) -> EditRequest {
        serde_json::from_value(json).unwrap()
    }

    fn coords() -> serde_json::Value {
        serde_json::json!({"lat": 48.262, "lon": 11.668})
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
        assert_eq!(req.extract_subject(), "add room `5117.EG.103`");
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
        let subj = req.extract_subject();
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
        assert_eq!(req.extract_subject(), "add 10 POIs");
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
        let subj = req.extract_subject();
        assert!(subj.contains("coordinate edit for `0101`"));
        assert!(subj.contains("add room `5117.EG.103`"));
        assert!(subj.contains(" and "));
    }
}
