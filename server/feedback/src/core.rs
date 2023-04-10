use crate::tokens::Claims;
use actix_web::web::Data;
use actix_web::HttpResponse;
use jsonwebtoken::{encode, EncodingKey, Header};
use log::error;

use tokio::sync::Mutex;

pub struct AppStateFeedback {
    pub(crate) token_record: Mutex<Vec<TokenRecord>>,
}
impl AppStateFeedback {
    pub fn new() -> AppStateFeedback {
        AppStateFeedback {
            token_record: Mutex::new(Vec::new()),
        }
    }
    pub fn able_to_process_feedback(&self) -> bool {
        std::env::var("GITHUB_TOKEN").is_ok() && std::env::var("JWT_KEY").is_ok()
    }
}

pub struct TokenRecord {
    pub kid: u64,
    pub next_reset: usize,
}

pub async fn get_token(state: Data<AppStateFeedback>) -> HttpResponse {
    if !state.able_to_process_feedback() {
        return HttpResponse::ServiceUnavailable()
            .content_type("text/plain")
            .body("Feedback is currently not configured on this server.");
    }

    let secret = std::env::var("JWT_KEY").unwrap(); // we checked the ability to process feedback
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
