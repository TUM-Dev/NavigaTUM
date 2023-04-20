use crate::github;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;

use crate::tokens::RecordedTokens;
use actix_web::post;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FeedbackPostData {
    token: String,
    category: String,
    subject: String,
    body: String,
    privacy_checked: bool,
    deletion_requested: bool,
}

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
    let (title_category, labels) = parse_request(&req_data);

    github::open_issue(
        &format!("[{title_category}] {subject}", subject = req_data.subject),
        &req_data.body,
        labels,
    )
    .await
}

fn parse_request(req_data: &Json<FeedbackPostData>) -> (&str, Vec<String>) {
    let title_category = match req_data.category.as_str() {
        "general" => "General",
        "bug" => "Bug",
        "feature" => "Feature",
        "search" => "Search",
        "entry" => "Entry",
        _ => "Form",
    };

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
    (title_category, labels)
}
