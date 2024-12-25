use actix_web::post;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

use super::github;
use super::tokens::RecordedTokens;
#[expect(
    unused_imports,
    reason = "has to be imported as otherwise utoipa generates incorrect code"
)]
use url::Url;

#[derive(Deserialize, Serialize, Default, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
enum FeedbackCategory {
    Bug,
    Feature,
    Search,
    Navigation,
    Entry,
    General,
    #[default]
    Other,
}
impl std::fmt::Display for FeedbackCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = serde_json::to_string(self).expect("FeedbackCategory is always serialisable");
        f.write_str(&val)
    }
}

#[derive(Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct FeedbackPostData {
    /// The JWT token, that can be used to generate feedback
    #[schema(
        example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2Njk2MzczODEsImlhdCI6MTY2OTU5NDE4MSwibmJmIjoxNjY5NTk0MTkxLCJraWQiOjE1ODU0MTUyODk5MzI0MjU0Mzg2fQ.sN0WwXzsGhjOVaqWPe-Fl5x-gwZvh28MMUM-74MoNj4"
    )]
    token: String,
    /// The category of the feedback.
    #[schema(example=FeedbackCategory::Bug)]
    #[serde(default)]
    category: FeedbackCategory,
    /// The subject/title of the feedback
    ///
    /// Controll characters will be stripped, too long input truncated and newlines made to render in markdown
    #[schema(example = "A catchy title", max_length = 512, min_length = 4)]
    subject: String,
    /// The body/description of the feedback
    ///
    /// Controll characters will be stripped, too long input truncated and newlines made to render in markdown
    #[schema(
        example = "A clear description what happened where and how we should improve it",
        max_length = 1048576,
        min_length = 10
    )]
    body: String,
    /// Whether the user has checked the privacy-checkbox.
    ///
    /// We are posting the feedback publicly on GitHub (not a EU-Company).
    /// **You MUST also include such a checkmark.**
    privacy_checked: bool,
    /// Whether the user has requested to delete the issue.
    ///
    /// This flag means:
    /// - If the user has requested to delete the issue, we will delete it from GitHub after processing it
    /// - If the user has not requested to delete the issue, we will not delete it from GitHub and it will remain as a closed issue.
    deletion_requested: bool,
}

/// Post feedback
///
/// ***Do not abuse this endpoint.***
///
/// This posts the actual feedback to GitHub and returns the GitHub link.
/// This API will create issues instead of pull-requests
/// => all feedback is allowed, but [`/api/feedback/propose_edits`](#tag/feedback/operation/propose_edits) is preferred, if it can be posted there.
///
/// For this Endpoint to work, you need to generate a token via the [`/api/feedback/get_token`](#tag/feedback/operation/get_token) endpoint.
///
/// # Note
///
/// Tokens are only used if we return a 201 Created response.
/// Otherwise, they are still valid
#[utoipa::path(
    tags=["feedback"],
    responses(
        (status = 201, description = "The feedback has been **successfully posted to GitHub**. We return the link to the GitHub issue.", body = Url, content_type = "text/plain", example = "https://github.com/TUM-Dev/navigatum/issues/9"),
        (status = 400, description = "**Bad Request.** Not all fields in the body are present as defined above"),
        (status = 403, description = r#"**Forbidden.** Causes are (delivered via the body):

- `Invalid token`: You have not supplied a token generated via the `gen_token`-Endpoint.
- `Token not old enough, please wait`: Tokens are only valid after 10s.
- `Token expired`: Tokens are only valid for 12h.
- `Token already used`: Tokens are non reusable/refreshable single-use items."#, body = String, content_type = "text/plain"),
        (status = 422, description = "**Unprocessable Entity.** Subject or body missing or too short."),
        (status = 451, description = "**Unavailable for legal reasons.** Using this endpoint without accepting the privacy policy is not allowed. For us to post to GitHub, this has to be `true`"),
        (status = 500, description = "**Internal Server Error.** We have a problem communicating with GitHubs servers. Please try again later"),
        (status = 503, description = "**Service unavailable.** We have not configured a GitHub Access Token. This could be because we are experiencing technical difficulties or intentional. Please try again later."),
    )
)]
#[post("/api/feedback/feedback")]
pub async fn send_feedback(
    recorded_tokens: Data<RecordedTokens>,
    req_data: Json<FeedbackPostData>,
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

    github::open_issue(&req_data.subject, &req_data.body, parse_labels(&req_data.0)).await
}

fn parse_labels(req_data: &FeedbackPostData) -> Vec<String> {
    let mut labels = vec!["webform".to_string()];
    if req_data.deletion_requested {
        labels.push("delete-after-processing".to_string());
    }
    labels.push(req_data.category.to_string());
    labels
}
