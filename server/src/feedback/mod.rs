mod github;
mod tokens;
use crate::feedback::tokens::{Claims, RateLimit};
use actix_web::web::{Data, Json};
use actix_web::{post, web, HttpResponse};
use jsonwebtoken::{encode, EncodingKey, Header};
use log::error;
use serde::Deserialize;

use tokio::sync::Mutex;

pub struct AppStateFeedback {
    available: bool,
    opt: crate::Opt,
    generated_tokens: RateLimit,
    consumed_tokens: RateLimit,
    token_record: Mutex<Vec<TokenRecord>>,
}

pub struct TokenRecord {
    kid: u64,
    next_reset: usize,
}

#[derive(Deserialize)]
struct FeedbackPostData {
    token: String,
    category: String,
    subject: String,
    body: String,
    privacy_checked: bool,
    delete_issue_requested: bool,
}

pub fn init_state(opt: crate::Opt) -> AppStateFeedback {
    let available = opt.github_token.is_some();
    AppStateFeedback {
        available,
        opt,
        generated_tokens: RateLimit::new(),
        consumed_tokens: RateLimit::new(),
        token_record: Mutex::new(Vec::new()),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_token).service(send_feedback);
}

#[post("/get_token")]
async fn get_token(state: Data<AppStateFeedback>) -> HttpResponse {
    //auth
    if !state.available {
        return HttpResponse::ServiceUnavailable()
            .content_type("text/plain")
            .body("Feedback is currently not configured on this server.");
    }
    if !state.generated_tokens.check_and_increment() {
        return HttpResponse::TooManyRequests()
            .content_type("text/plain")
            .body("Too many tokens generated. Please try again later.");
    }

    // we now know that we are allowed to generate a token

    let token = encode(
        &Header::default(),
        &Claims::new(),
        &EncodingKey::from_secret("secret".as_ref()),
    );

    match token {
        Ok(token) => HttpResponse::Ok().json(token),
        Err(e) => {
            error!("Failed to generate token: {:?}", e);
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Failed to generate token.")
        }
    }
}

#[post("/feedback")]
async fn send_feedback(
    state: Data<AppStateFeedback>,
    req_data: Json<FeedbackPostData>,
) -> HttpResponse {
    //auth
    let maybe_err = tokens::validate_token(&state, &req_data.token).await;
    if let Some(e) = maybe_err {
        return e;
    }

    // validate request
    if !req_data.privacy_checked {
        return HttpResponse::UnavailableForLegalReasons()
            .content_type("text/plain")
            .body("Using this endpoint without accepting the privacy policy is not allowed");
    };
    let (title_category, labels) = parse_request(&req_data);

    let github_token = state.opt.github_token.as_ref().unwrap().trim().to_string();
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
