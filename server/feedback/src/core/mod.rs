mod github;
mod tokens;
use crate::core::tokens::Claims;
use actix_web::web::{Data, Json};
use actix_web::{post, HttpResponse};
use jsonwebtoken::{encode, EncodingKey, Header};
use log::error;
use serde::Deserialize;

use tokio::sync::Mutex;

pub struct AppStateFeedback {
    feedback_keys: crate::FeedbackKeys,
    token_record: Mutex<Vec<TokenRecord>>,
}
impl AppStateFeedback {
    pub fn from(feedback_keys: crate::FeedbackKeys) -> AppStateFeedback {
        AppStateFeedback {
            feedback_keys,
            token_record: Mutex::new(Vec::new()),
        }
    }
    pub fn able_to_process_feedback(&self) -> bool {
        self.feedback_keys.github_token.is_some() && self.feedback_keys.jwt_key.is_some()
    }
}

pub struct TokenRecord {
    kid: u64,
    next_reset: usize,
}

#[derive(Deserialize)]
pub struct FeedbackPostData {
    token: String,
    category: String,
    subject: String,
    body: String,
    privacy_checked: bool,
    delete_issue_requested: bool,
}

pub async fn get_token(state: Data<AppStateFeedback>) -> HttpResponse {
    if !state.able_to_process_feedback() {
        return HttpResponse::ServiceUnavailable()
            .content_type("text/plain")
            .body("Feedback is currently not configured on this server.");
    }

    let secret = state.feedback_keys.jwt_key.clone().unwrap(); // we checked available
    let token = encode(
        &Header::default(),
        &Claims::new(),
        &EncodingKey::from_secret(secret.as_bytes()),
    );

    match token {
        Ok(token) => HttpResponse::Created().json(token),
        Err(e) => {
            error!("Failed to generate token: {e:?}");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Failed to generate token.")
        }
    }
}

#[post("/api/feedback/feedback")]
pub async fn send_feedback(
    state: Data<AppStateFeedback>,
    req_data: Json<FeedbackPostData>,
) -> HttpResponse {
    // auth
    if let Some(e) = tokens::validate_token(&state, &req_data.token).await {
        return e;
    }

    // validate request
    if !req_data.privacy_checked {
        return HttpResponse::UnavailableForLegalReasons()
            .content_type("text/plain")
            .body("Using this endpoint without accepting the privacy policy is not allowed");
    };
    let (title_category, labels) = parse_request(&req_data);

    let github_token = state
        .feedback_keys
        .github_token
        .as_ref()
        .unwrap()
        .trim()
        .to_string();
    github::post_feedback(
        github_token,
        title_category,
        &req_data.subject,
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
    if req_data.delete_issue_requested {
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
