use actix_web::{post, web, HttpResponse};
use awc::Client;
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
    privacy: String,
}

#[derive(Serialize)]
struct CreateIssuePostData {
    title: String,
    description: String,
    labels: String,
    confidential: bool,
}

#[derive(Serialize)]
struct GenerateTokenResult {
    token: String,
}

pub fn init_state(opt: crate::Opt) -> AppStateFeedback {
    let available =
        opt.gitlab_domain.is_some() && opt.gitlab_token.is_some() && opt.feedback_project.is_some();

    AppStateFeedback {
        available: available,
        opt: opt,
        token: Mutex::new(Vec::<Token>::new()),
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
            .rposition(|t| t.creation.elapsed().as_secs() > 3600 * 1)
            .unwrap_or_else(|| 0);

    if token.len() >= RATE_LIMIT_DAY || num_token_last_hour >= RATE_LIMIT_HOUR {
        HttpResponse::TooManyRequests()
            .content_type("plain/text")
            .body("Too many token generated recently. Please try again later.".to_string())
    } else {
        // Simple numbers as random token for now
        let mut rng = thread_rng();
        let token_value: i64 = rng.gen_range(100_000_000_000_000..999_999_999_999_999);

        let new_token = Token {
            value: token_value.to_string(),
            creation: Instant::now(),
            used: false,
        };

        token.push(new_token);
        HttpResponse::Created().json(GenerateTokenResult {
            token: token_value.to_string(),
        })
    }
}

#[post("/feedback")]
async fn send_feedback(
    state: web::Data<AppStateFeedback>,
    req_data: web::Json<FeedbackPostData>,
) -> HttpResponse {
    if !state.available {
        return HttpResponse::ServiceUnavailable()
            .content_type("plain/text")
            .body("Feedback is currently not configured on this server.");
    }

    let mut token_list = state.token.lock().unwrap();

    let token = token_list.iter_mut().find(|t| t.value == req_data.token);

    if let Some(t) = token {
        if t.creation.elapsed().as_secs() < TOKEN_MIN_AGE {
            return HttpResponse::Forbidden()
                .content_type("plain/text")
                .body("Token not old enough, please wait.");
        } else if t.creation.elapsed().as_secs() > TOKEN_MAX_AGE {
            return HttpResponse::Forbidden()
                .content_type("plain/text")
                .body("Token expired.");
        } else if t.used {
            return HttpResponse::Forbidden()
                .content_type("plain/text")
                .body("Token already used.");
        }

        let post_data = CreateIssuePostData {
            title: format!("Form: {}", clean_feedback_data(&req_data.subject, 512)),
            description: clean_feedback_data(&req_data.body, 1024 * 1024).replace("/", "//"), // Do not use GitLab quick actions
            labels: format!(
                "webform,{}",
                match req_data.category.as_str() {
                    "general" | "bug" | "features" | "search" | "entry" => &req_data.category,
                    _ => "other",
                }
            ),
            confidential: req_data.privacy.as_str() == "internal",
        };

        if post_data.title.len() < 3 || post_data.description.len() < 10 {
            return HttpResponse::UnprocessableEntity()
                .content_type("plain/text")
                .body("Subject or body missing or too short.");
        }

        let resp = Client::new()
            .post(format!(
                "https://{}/api/v4/projects/{}/issues",
                state.opt.gitlab_domain.as_ref().unwrap(),
                state.opt.feedback_project.as_ref().unwrap()
            ))
            .insert_header((
                "PRIVATE-TOKEN",
                state.opt.gitlab_token.as_ref().unwrap().to_string(),
            ))
            .send_json(&post_data)
            .await;

        resp.and_then(|resp| {
            if resp.status().is_success() {
                t.used = true;
                Ok(HttpResponse::Ok().body("Success".to_string()))
            } else {
                Ok(HttpResponse::InternalServerError().body("Failed to send".to_string()))
            }
        })
        .unwrap()
    } else {
        HttpResponse::Forbidden()
            .content_type("plain/text")
            .body("Invalid token".to_string())
    }
}

fn clean_feedback_data(s: &String, len: usize) -> String {
    if len > 0 {
        let mut s = s.clone();
        s.truncate(len);
        s.chars().filter(|c| !c.is_control()).collect()
    } else {
        s.chars().filter(|c| !c.is_control()).collect()
    }
}
