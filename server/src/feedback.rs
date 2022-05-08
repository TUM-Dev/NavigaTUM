use actix_web::web::Json;
use actix_web::{post, web, HttpResponse};
use log::error;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Instant;

extern crate rand;

use rand::thread_rng;
use rand::Rng;

// As a very basic rate limiting, the generation of tokens
// is limited to a fixed amount per day and hour.
const RATE_LIMIT_HOUR: usize = 20;
const RATE_LIMIT_DAY: usize = 50; // = 24h

// Additionally, there is a short delay until a token can be used.
// Clients need to wait that time if (for some reason) the user submitted
// faster than limited here.
const TOKEN_MIN_AGE: u64 = 10;
const TOKEN_MAX_AGE: u64 = 3600 * 12; // 12h

pub struct AppStateFeedback {
    available: bool,
    opt: crate::Opt,
    token: Mutex<Vec<Token>>,
}

#[derive(Debug)]
struct Token {
    value: String,
    creation: Instant,
    used: bool,
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

#[derive(Serialize)]
struct GenerateTokenResult {
    token: String,
}

pub fn init_state(opt: crate::Opt) -> AppStateFeedback {
    let available = opt.github_token.is_some();
    let token = Mutex::new(Vec::<Token>::new());
    AppStateFeedback {
        available,
        opt,
        token,
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_token).service(send_feedback);
}

#[post("/get_token")]
async fn get_token(state: web::Data<AppStateFeedback>) -> HttpResponse {
    if !state.available {
        return HttpResponse::ServiceUnavailable()
            .content_type("plain/text")
            .body("Feedback is currently not configured on this server.");
    }

    let mut token = state.token.lock().unwrap();

    // remove outdated token (no longer relevant for rate limit)
    token.retain(|t| t.creation.elapsed().as_secs() < 3600 * 24 && !t.used);

    let num_token_last_hour = token.len()
        - token
            .iter()
            .rposition(|t| t.creation.elapsed().as_secs() > 3600)
            .unwrap_or(0);

    if token.len() >= RATE_LIMIT_DAY || num_token_last_hour >= RATE_LIMIT_HOUR {
        return HttpResponse::TooManyRequests()
            .content_type("plain/text")
            .body("Too many token generated recently. Please try again later.");
    }
    // Simple numbers as random token for now
    let mut rng = thread_rng();
    let token_value: i64 = rng.gen_range(100_000_000_000_000..999_999_999_999_999);

    let new_token = Token {
        value: token_value.to_string(),
        creation: Instant::now(),
        used: false,
    };

    token.push(new_token);
    let token_result = GenerateTokenResult {
        token: token_value.to_string(),
    };
    HttpResponse::Created().json(token_result)
}

#[post("/feedback")]
async fn send_feedback(
    state: web::Data<AppStateFeedback>,
    req_data: web::Json<FeedbackPostData>,
) -> HttpResponse {
    if !state.available {
        return HttpResponse::ServiceUnavailable()
            .content_type("text/plain")
            .body("Feedback is currently not configured on this server.");
    }

    let mut token_list = state.token.lock().unwrap();

    let token = token_list.iter_mut().find(|t| t.value == req_data.token);

    if token.is_none() {
        return HttpResponse::Forbidden()
            .content_type("text/plain")
            .body("Invalid token");
    }
    let t = token.unwrap();
    if t.creation.elapsed().as_secs() < TOKEN_MIN_AGE {
        return HttpResponse::Forbidden()
            .content_type("text/plain")
            .body("Token not old enough, please wait.");
    }
    if t.creation.elapsed().as_secs() > TOKEN_MAX_AGE {
        return HttpResponse::Forbidden()
            .content_type("text/plain")
            .body("Token expired.");
    }
    if t.used {
        return HttpResponse::Forbidden()
            .content_type("text/plain")
            .body("Token already used.");
    }
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
    let token = state.opt.github_token.as_ref().unwrap().to_string();
    let octocrab = Octocrab::builder().personal_token(token).build();
    if octocrab.is_err() {
        error!("Error creating issue: {:?}", octocrab);
        return HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body("Could not create Octocrab instance");
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
        Ok(issue) => {
            t.used = true;
            HttpResponse::Created()
                .content_type("text/plain")
                .body(issue.html_url.to_string())
        }
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

fn clean_feedback_data(s: &str, len: usize) -> String {
    if len > 0 {
        s.chars().filter(|c| !c.is_control()).take(len).collect()
    } else {
        s.chars().filter(|c| !c.is_control()).collect()
    }
}
