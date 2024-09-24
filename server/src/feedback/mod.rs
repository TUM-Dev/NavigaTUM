use actix_governor::{GlobalKeyExtractor, Governor, GovernorConfigBuilder};
use actix_web::web;

mod github;
mod post_feedback;
mod proposed_edits;
mod tokens;

const SECONDS_PER_DAY: u64 = 60 * 60 * 24;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let feedback_ratelimit = GovernorConfigBuilder::default()
        .key_extractor(GlobalKeyExtractor)
        .seconds_per_request(SECONDS_PER_DAY / 300) // replenish new token every .. seconds
        .burst_size(50)
        .finish()
        .expect("Invalid configuration of the governor");

    let recorded_tokens = web::Data::new(tokens::RecordedTokens::default());
    cfg.app_data(recorded_tokens.clone())
        .service(post_feedback::send_feedback)
        .service(proposed_edits::propose_edits)
        .service(
            web::scope("/get_token")
                .wrap(Governor::new(&feedback_ratelimit))
                .route("", web::post().to(tokens::get_token)),
        );
}
