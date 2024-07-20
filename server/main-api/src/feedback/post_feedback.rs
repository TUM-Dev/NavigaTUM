use actix_web::post;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::Deserialize;

use super::github;
use super::tokens::RecordedTokens;

#[derive(Deserialize)]
pub struct FeedbackPostData {
    token: String,
    category: String,
    subject: String,
    body: String,
    privacy_checked: bool,
    deletion_requested: bool,
}

#[post("/feedback")]
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

    github::open_issue(&req_data.subject, &req_data.body, parse_labels(&req_data)).await
}

fn parse_labels(req_data: &Json<FeedbackPostData>) -> Vec<String> {
    let mut labels = vec!["webform".to_string()];
    if req_data.deletion_requested {
        labels.push("delete-after-processing".to_string());
    }
    match req_data.category.as_str() {
        "general" | "bug" | "feature" | "search" | "entry" => {
            labels.push(req_data.category.as_str().to_string());
        }
        _ => {}
    };
    labels
}
