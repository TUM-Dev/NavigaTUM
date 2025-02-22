use std::fmt;

use actix_web::{HttpResponse, post};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::error;

#[derive(Default)]
pub struct RecordedTokens(Mutex<Vec<TokenRecord>>);

impl fmt::Debug for RecordedTokens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //fields purposely omitted
        f.debug_struct("RecordedTokens").finish()
    }
}

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

impl Default for Claims {
    fn default() -> Self {
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
    #[tracing::instrument(skip(token))]
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
                error!(kind=?e.kind(),"Failed to decode token");
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

#[derive(Debug, Serialize, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
struct TokenResponse {
    /// Unix timestamp of when the token was created
    #[schema(example = "1629564181")]
    created_at: i64,
    /// The JWT token, that can be used to generate feedback
    #[schema(
        example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2Njk2MzczODEsImlhdCI6MTY2OTU5NDE4MSwibmJmIjoxNjY5NTk0MTkxLCJraWQiOjE1ODU0MTUyODk5MzI0MjU0Mzg2fQ.sN0WwXzsGhjOVaqWPe-Fl5x-gwZvh28MMUM-74MoNj4"
    )]
    token: String,
}

/// Get a feedback-token
///
/// ***Do not abuse this endpoint.***
///
/// This returns a JWT token usable for submitting feedback.
/// You should request a token, ***if (and only if) a user is on a feedback page***
///
/// As a rudimentary way of rate-limiting feedback, this endpoint returns a token.
/// To post feedback, you will need this token.
///
/// Tokens gain validity after 5s, and are invalid after 12h of being issued.
/// They are not refreshable, and are only valid for one usage.
///
/// # Note:
///
/// Global Rate-Limiting allows bursts with up to 20 requests and replenishes 50 requests per day
#[utoipa::path(
    tags=["feedback"],
    responses(
        (status = 201, description = "**Created** a usable token", body= TokenResponse, content_type="application/json"),
        (status = 429, description = "**Too many requests.** We are rate-limiting everyone's requests, please try again later."),
        (status = 503, description= "**Service unavailable.** We have not configured a GitHub Access Token. This could be because we are experiencing technical difficulties or intentional. Please try again later."),
    )
)]
#[post("")]
pub async fn get_token() -> HttpResponse {
    if !able_to_process_feedback() {
        return HttpResponse::ServiceUnavailable()
            .content_type("text/plain")
            .body("Feedback is currently not configured on this server.");
    }

    let secret = std::env::var("JWT_KEY").unwrap(); // we checked the ability to process feedback
    let token = encode(
        &Header::default(),
        &Claims::default(),
        &EncodingKey::from_secret(secret.as_bytes()),
    );

    match token {
        Ok(token) => {
            let created_at = chrono::Utc::now().timestamp();
            HttpResponse::Created().json(TokenResponse { created_at, token })
        }
        Err(e) => {
            error!(error = ?e, "Failed to generate token");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Failed to generate token, please try again later")
        }
    }
}
