use actix_web::web::{Data, Json};
use actix_web::{post, web, HttpResponse};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use log::error;
use octocrab::Octocrab;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    nbf: usize, // Optional. Not Before (as UTC timestamp)
    kid: u64,   // Optional. Key ID
}

// As a very basic rate limiting, the generation of tokens
// is limited to a fixed amount per day and hour.
const RATE_LIMIT_HOUR: u64 = 20;
const RATE_LIMIT_DAY: u64 = 50; // = 24h

// Additionally, there is a short delay until a token can be used.
// Clients need to wait that time if (for some reason) the user submitted
// faster than limited here.
const TOKEN_MIN_AGE: usize = 10;
const TOKEN_MAX_AGE: usize = 3600 * 12; // 12h

struct Rate {
    cnt: AtomicU64,
    next_reset: AtomicUsize,
    limit: u64,
    increment_reset: usize,
}

struct RateLimit {
    hour: Rate,
    day: Rate,
}

impl Rate {
    fn new(limit: u64, increment_reset: usize) -> Rate {
        Rate {
            cnt: AtomicU64::new(0),
            next_reset: AtomicUsize::new(0),
            limit,
            increment_reset,
        }
    }
    fn check_and_increment(&self) -> bool {
        // Returns true if the value was incremented.
        // Returns false if the limit was reached.
        // Uses CMPXCHG to ensure that this is theadsave
        let now = chrono::Utc::now().timestamp() as usize;
        if self.next_reset.load(Ordering::SeqCst) < now {
            // Reset the counter, as the time since the last reset is as long as the reset interval
            self.cnt.store(1, Ordering::SeqCst);
            self.next_reset
                .store(now + self.increment_reset, Ordering::SeqCst);
            return true;
        }
        let mut old = self.cnt.load(Ordering::SeqCst);
        loop {
            let new = old + 1;
            if old >= self.limit {
                return false;
            }
            match self
                .cnt
                .compare_exchange_weak(old, new, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return true,
                Err(x) => old = x,
            }
        }
    }
}

impl RateLimit {
    fn new() -> RateLimit {
        RateLimit {
            hour: Rate::new(RATE_LIMIT_HOUR, 3600),
            day: Rate::new(RATE_LIMIT_DAY, 3600 * 24),
        }
    }
    fn check_and_increment(&self) -> bool {
        self.day.check_and_increment() && self.hour.check_and_increment()
    }
}

struct TokenRecord {
    kid: u64,
    next_reset: usize,
}

pub struct AppStateFeedback {
    available: bool,
    opt: crate::Opt,
    generated_tokens: RateLimit,
    consumed_tokens: RateLimit,
    token_record: Mutex<Vec<TokenRecord>>,
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

    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        exp: now + TOKEN_MAX_AGE,
        iat: now,
        nbf: now + TOKEN_MIN_AGE,
        kid: rand::random(),
    };
    let token = encode(
        &Header::default(),
        &claims,
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
    validate_token(&state, &req_data).await;

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

async fn validate_token(
    state: &Data<AppStateFeedback>,
    req_data: &Json<FeedbackPostData>,
) -> Option<HttpResponse> {
    if !state.available {
        return Some(
            HttpResponse::ServiceUnavailable()
                .content_type("text/plain")
                .body("Feedback is currently not configured on this server."),
        );
    }
    if !state.consumed_tokens.check_and_increment() {
        return Some(
            HttpResponse::TooManyRequests()
                .content_type("text/plain")
                .body("Too many tokens generated. Please try again later."),
        );
    }

    let x = DecodingKey::from_secret("secret".as_ref());
    let jwt_token = decode::<Claims>(&req_data.token, &x, &Validation::default());
    let kid = match jwt_token {
        Ok(token) => token.claims.kid,
        Err(e) => {
            error!("Failed to decode token: {:?}", e.kind());
            return Some(HttpResponse::Forbidden().content_type("text/plain").body(
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ImmatureSignature => "Token is not yet valid.",
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Token expired",
                    _ => "Invalid token",
                },
            ));
        }
    };

    // now we know that the token is valid and thus in time and created by us.
    // The problem is, that it could be used multiple times.
    // To prevent this, we need to check if the token was already used.

    let now = chrono::Utc::now().timestamp() as usize;
    let mut tokens = state.token_record.lock().await;
    // remove outdated tokens (no longer relevant for rate limit)
    tokens.retain(|t| t.next_reset > now + TOKEN_MAX_AGE);
    // check if token is already used
    if tokens.iter().any(|r| r.kid == kid) {
        return Some(
            HttpResponse::Forbidden()
                .content_type("text/plain")
                .body("Token already used."),
        );
    }
    None
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
