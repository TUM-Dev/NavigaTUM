mod tokens;

use actix_web::web::{Data, Json};
use actix_web::{post, web, HttpResponse};

use crate::feedback::tokens::{Claims, RateLimit};
use jsonwebtoken::encode;

use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;

use log::error;
use octocrab::Octocrab;
use regex::Regex;
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
        token_record: Mutex::new(Vec::<TokenRecord>::new()),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_token).service(send_feedback);
}

#[post("/get_token")]
async fn get_token(state: Data<AppStateFeedback>) -> HttpResponse {
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

    let (title, description, labels) = parse_request(&req_data);

    if title.len() < 3 || description.len() < 10 {
        return HttpResponse::UnprocessableEntity()
            .content_type("text/plain")
            .body("Subject or body missing or too short");
    }

    let github_token = state.opt.github_token.as_ref().unwrap().trim().to_string();
    post_feedback(github_token, title, description, labels).await
}

async fn post_feedback(
    github_token: String,
    title: String,
    description: String,
    labels: Vec<String>,
) -> HttpResponse {
    let octocrab = Octocrab::builder().personal_token(github_token).build();
    if octocrab.is_err() {
        error!("Error creating issue: {:?}", octocrab);
        return HttpResponse::InternalServerError().body("Could not create Octocrab instance");
    }
    let resp = octocrab
        .unwrap()
        .issues("TUM-Dev", "navigatum")
        .create(title)
        .body(description)
        .labels(labels)
        .send()
        .await;

    return match resp {
        Ok(issue) => HttpResponse::Created()
            .content_type("text/plain")
            .body(issue.html_url.to_string()),
        Err(e) => {
            error!("Error creating issue: {:?}", e);
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Failed create issue")
        }
    };
}

fn parse_request(req_data: &Json<FeedbackPostData>) -> (String, String, Vec<String>) {
    let capitalised_category = match req_data.category.as_str() {
        "general" => "General",
        "bug" => "Bug",
        "feature" => "Feature",
        "search" => "Search",
        "entry" => "Entry",
        _ => "Form",
    };
    let raw_title = format!("[{}] {}", capitalised_category, &req_data.subject);
    let title = clean_feedback_data(&raw_title, 512);
    let description = clean_feedback_data(&req_data.body, 1024 * 1024);

    let mut labels = vec![String::from("webform")];
    if req_data.delete_issue_requested {
        labels.push(String::from("delete-after-processing"));
    }
    match req_data.category.as_str() {
        "general" | "bug" | "feature" | "search" | "entry" => {
            labels.push(String::from(&req_data.category));
        }
        _ => {}
    };
    (title, description, labels)
}

/// Remove all returns a string, which has
/// - all control characters removed
/// - is at most len characters long
/// - can be nicely formatted in markdown (just \n in md is not a linebreak)
fn clean_feedback_data(s: &str, len: usize) -> String {
    let s_clean = s
        .chars()
        .filter(|c| !c.is_control() || (c == &'\n'))
        .take(len)
        .collect::<String>();

    let re = Regex::new(r"[ \t]*\n").unwrap();
    re.replace_all(&s_clean, "  \n").to_string()
}

#[cfg(test)]
mod description_tests {
    use super::*;

    #[test]
    fn newlines_whitespace() {
        assert_eq!(
            clean_feedback_data("a\r\nb", 9),
            clean_feedback_data("a\nb", 9)
        );
        assert_eq!(clean_feedback_data("a\nb\nc", 9), "a  \nb  \nc");
        assert_eq!(clean_feedback_data("a\nb  \nc", 9), "a  \nb  \nc");
        assert_eq!(clean_feedback_data("a      \nb", 9), "a  \nb");
        assert_eq!(clean_feedback_data("a\n\nb", 9), "a  \n  \nb");
        assert_eq!(clean_feedback_data("a\n   b", 9), "a  \n   b");
    }
    #[test]
    fn truncate_len() {
        for i in 0..10 {
            let mut expected = "abcd".to_string();
            expected.truncate(i);
            assert_eq!(clean_feedback_data("abcd", i), expected);
        }
    }
    #[test]
    fn special_cases() {
        assert_eq!(clean_feedback_data("", 0), "");
        assert_eq!(clean_feedback_data("a\x05bc", 9), "abc");
        assert_eq!(clean_feedback_data("ab\x0Dc", 9), "abc");
    }
}
