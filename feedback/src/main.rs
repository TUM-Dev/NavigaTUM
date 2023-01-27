use actix_cors::Cors;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer};

use structopt::StructOpt;

mod core;

const MAX_JSON_PAYLOAD: usize = 1024 * 1024; // 1 MB

#[derive(StructOpt, Debug)]
#[structopt(name = "server")]
pub struct Opt {
    // Feedback
    /// GitHub personal access token
    #[structopt(short = "t", long)]
    github_token: Option<String>,
    /// Secret for the feedback token generation
    #[structopt(short = "jwt", long)]
    jwt_key: Option<String>,
}

#[get("/api/feedback/health")]
async fn health_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain")
        .body("healthy")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let mut opt = Opt::from_args();
    if opt.github_token.is_none() {
        opt.github_token = std::env::var("GITHUB_TOKEN").ok();
    }
    if opt.jwt_key.is_none() {
        opt.jwt_key = std::env::var("JWT_KEY").ok();
    }

    let state_feedback = web::Data::new(core::init_state(opt));
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allowed_methods(vec!["GET", "POST"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default().exclude("/api/feedback/health"))
            .wrap(middleware::Compress::default())
            .app_data(web::JsonConfig::default().limit(MAX_JSON_PAYLOAD))
            .service(health_handler)
            .service(
                web::scope("/api/feedback")
                    .configure(core::configure)
                    .app_data(state_feedback.clone()),
            )
    })
    .bind(std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8070".to_string()))?
    .run()
    .await
}
