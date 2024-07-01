use actix_web::HttpResponse;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::error;

#[derive(Default)]
pub struct RecordedTokens(Mutex<Vec<TokenRecord>>);

pub struct TokenRecord {
    kid: u64,
    next_reset: i64,
}

fn able_to_process_feedback() -> bool {
    std::env::var("GITHUB_TOKEN").is_ok() && std::env::var("JWT_KEY").is_ok()
}

// Additionally, there is a short delay until a token can be used.
// Clients need to wait that time if (for some reason) the user submitted
// faster than limited here.
const TOKEN_MIN_AGE: i64 = 5;
const TOKEN_MAX_AGE: i64 = 3600 * 12; // 12h

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: i64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: i64, // Optional. Issued at (as UTC timestamp)
    nbf: i64, // Optional. Not Before (as UTC timestamp)
    kid: u64, // Optional. Key ID
}

impl Claims {
    pub fn new() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            exp: now + TOKEN_MAX_AGE,
            iat: now,
            nbf: now + TOKEN_MIN_AGE,
            kid: rand::random(),
        }
    }
}

impl RecordedTokens {
    pub async fn validate(&self, token: &str) -> Option<HttpResponse> {
        if !able_to_process_feedback() {
            return Some(
                HttpResponse::ServiceUnavailable()
                    .content_type("text/plain")
                    .body("Feedback is currently not configured on this server."),
            );
        }

        let secret = std::env::var("JWT_KEY").unwrap(); // we checked the ability to process feedback
        let x = DecodingKey::from_secret(secret.as_bytes());
        let jwt_token = decode::<Claims>(token, &x, &Validation::default());
        let kid = match jwt_token {
            Ok(token) => token.claims.kid,
            Err(e) => {
                error!("Failed to decode token: {:?}", e.kind());
                return Some(HttpResponse::Forbidden().content_type("text/plain").body(
                    match e.kind() {
                        jsonwebtoken::errors::ErrorKind::ImmatureSignature => {
                            "Token is not yet valid."
                        }
                        jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Token expired",
                        _ => "Invalid token",
                    },
                ));
            }
        };

        // now we know from token-validity, that it is within our time limits and created by us.
        // The problem is, that it could be used multiple times.
        // To prevent this, we need to check if the token was already used.
        // This is means that if this usage+our ratelimits are
        // - neither synced across multiple feedback instances, nor
        // - persisted between reboots

        let now = chrono::Utc::now().timestamp();
        let mut tokens = self.0.lock().await;
        // remove outdated tokens (no longer relevant for rate limit)
        tokens.retain(|t| t.next_reset > now);
        // check if token is already used
        if tokens.iter().any(|r| r.kid == kid) {
            return Some(
                HttpResponse::Forbidden()
                    .content_type("text/plain")
                    .body("Token already used."),
            );
        }
        tokens.push(TokenRecord {
            kid,
            next_reset: now + TOKEN_MAX_AGE,
        });
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Token {
    created_at: i64, // unix timestamp
    token: String,
}

pub async fn get_token() -> HttpResponse {
    if !able_to_process_feedback() {
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
        Ok(token) => {
            let created_at = chrono::Utc::now().timestamp();
            HttpResponse::Created().json(Token { created_at, token })
        }
        Err(e) => {
            error!("Failed to generate token: {e:?}");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Failed to generate token.")
        }
    }
}
