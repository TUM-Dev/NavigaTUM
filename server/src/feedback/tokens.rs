use actix_web::web::Data;
use actix_web::HttpResponse;

use jsonwebtoken::{decode, DecodingKey, Validation};
use log::error;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use crate::feedback::{AppStateFeedback, TokenRecord};

// Additionally, there is a short delay until a token can be used.
// Clients need to wait that time if (for some reason) the user submitted
// faster than limited here.
const TOKEN_MIN_AGE: usize = 10;
const TOKEN_MAX_AGE: usize = 3600 * 12; // 12h

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    nbf: usize, // Optional. Not Before (as UTC timestamp)
    kid: u64,   // Optional. Key ID
}

impl Claims {
    pub fn new() -> Self {
        let now = chrono::Utc::now().timestamp() as usize;
        Self {
            exp: now + TOKEN_MAX_AGE,
            iat: now,
            nbf: now + TOKEN_MIN_AGE,
            kid: rand::random(),
        }
    }
}

// As a very basic rate limiting, the generation of tokens
// is limited to a fixed amount per day and hour.
const RATE_LIMIT_HOUR: u64 = 20;
const RATE_LIMIT_DAY: u64 = 50; // = 24h

struct Rate {
    cnt: AtomicU64,
    next_reset: AtomicUsize,
    limit: u64,
    increment_reset: usize,
}

pub struct RateLimit {
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
    pub(crate) fn new() -> RateLimit {
        RateLimit {
            hour: Rate::new(RATE_LIMIT_HOUR, 3600),
            day: Rate::new(RATE_LIMIT_DAY, 3600 * 24),
        }
    }
    pub(crate) fn check_and_increment(&self) -> bool {
        self.day.check_and_increment() && self.hour.check_and_increment()
    }
}

pub async fn validate_token(
    state: &Data<AppStateFeedback>,
    supplied_token: &str,
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
    let jwt_token = decode::<Claims>(supplied_token, &x, &Validation::default());
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
    tokens.push(TokenRecord {
        kid,
        next_reset: now + TOKEN_MAX_AGE,
    });
    None
}
