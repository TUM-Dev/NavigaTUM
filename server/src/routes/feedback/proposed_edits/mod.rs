use std::collections::HashMap;
use std::path::Path;

use actix_web::web::{Data, Json};
use actix_web::{HttpResponse, post};
use serde::Deserialize;
use tracing::error;
#[expect(
    unused_imports,
    reason = "has to be imported as otherwise utoipa generates incorrect code"
)]
use url::Url;

use crate::limited::hash_map::LimitedHashMap;

use super::proposed_edits::coordinate::Coordinate;
use super::proposed_edits::image::Image;
use super::proposed_edits::tmp_repo::TempRepo;
use super::tokens::RecordedTokens;
use crate::external::github::GitHub;

mod coordinate;
mod discription;
mod image;
mod tmp_repo;

#[derive(Debug, Deserialize, Clone, utoipa::ToSchema)]
struct Edit {
    coordinate: Option<Coordinate>,
    image: Option<Image>,
}
pub trait AppliableEdit {
    fn apply(&self, key: &str, base_dir: &Path) -> String;
}

#[derive(Debug, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct EditRequest {
    /// The JWT token, that can be used to generate feedback
    #[schema(
        example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2Njk2MzczODEsImlhdCI6MTY2OTU5NDE4MSwibmJmIjoxNjY5NTk0MTkxLCJraWQiOjE1ODU0MTUyODk5MzI0MjU0Mzg2fQ.sN0WwXzsGhjOVaqWPe-Fl5x-gwZvh28MMUM-74MoNj4"
    )]
    token: String,
    /// The edits to be made to the room. The keys are the ID of the props to be edited, the values are the proposed Edits.
    edits: LimitedHashMap<String, Edit>,
    /// Additional context for the edit.
    ///
    /// Will be displayed in the discription field of the PR
    #[schema(example = "I have a picture of the room, please add it to the roomfinder")]
    additional_context: String,
    /// Whether the user has checked the privacy-checkbox.
    ///
    /// We are posting the feedback publicly on GitHub (not a EU-Company).
    /// **You MUST also include such a checkmark.**
    privacy_checked: bool,
}

const GIT_URL: &str = "git@github.com:TUM-Dev/NavigaTUM.git";
impl EditRequest {
    #[tracing::instrument]
    async fn apply_changes_and_generate_description(
        &self,
        branch_name: &str,
    ) -> anyhow::Result<String> {
        let repo = TempRepo::clone_and_checkout(GIT_URL, branch_name).await?;
        let desc = repo.apply_and_gen_description(self);
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

    fn extract_labels(&self) -> Vec<String> {
        let mut labels = vec!["webform".to_string()];

        if self
            .edits
            .0
            .iter()
            .any(|(_, edit)| edit.coordinate.is_none())
        {
            labels.push("coordinate".to_string());
        }
        if self.edits.0.iter().any(|(_, edit)| edit.image.is_none()) {
            labels.push("image".to_string());
        }
        labels
    }
    fn extract_subject(&self) -> String {
        let coordinate_edits = self.edits_for(|edit| edit.coordinate);
        let image_edits = self.edits_for(|edit| edit.image);
        match (coordinate_edits.len(), image_edits.len()) {
            (0, 0) => "No Edits".to_string(),
            (1..=5, 0) => format!("Coordinate Edit for {:?}", coordinate_edits.keys()),
            (0, 1..=5) => format!("Added Images for {:?}", image_edits.keys()),
            (0, is) => format!("Added {is} Images"),
            (1..=3, 1..=3) => format!(
                "Edited Images for {:?} and Coordinates for {:?}",
                image_edits.keys(),
                coordinate_edits.keys()
            ),
            (cs, 0) => format!("Edited {cs} Coordinates"),
            (cs, is) => format!("Edited {is} Images and {cs} Coordinates"),
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
        (status = 403, description= r#"**Forbidden.** Causes are (delivered via the body):

- `Invalid token`: You have not supplied a token generated via the `gen_token`-Endpoint.
- `Token not old enough, please wait`: Tokens are only valid after 10s.
- `Token expired`: Tokens are only valid for 12h.
- `Token already used`: Tokens are non reusable/refreshable single-use items."#),
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
    };
    if req_data.edits.0.is_empty() {
        return HttpResponse::UnprocessableEntity()
            .content_type("text/plain")
            .body("Not enough edits provided");
    };
    if req_data.edits.0.len() > 500 {
        return HttpResponse::InsufficientStorage()
            .content_type("text/plain")
            .body("Too many edits provided");
    };

    let branch_name = format!("usergenerated/request-{}", rand::random::<u16>());
    match req_data
        .apply_changes_and_generate_description(&branch_name)
        .await
    {
        Ok(description) => {
            GitHub::default()
                .open_pr(
                    branch_name,
                    &format!(
                        "[User-Provided] {subject}",
                        subject = req_data.extract_subject()
                    ),
                    &description,
                    req_data.extract_labels(),
                )
                .await
        }
        Err(error) => {
            error!(?error, "could not apply changes");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Could apply changes, please try again later")
        }
    }
}
