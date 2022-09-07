use actix_cors::Cors;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer};
use log::warn;

use structopt::StructOpt;

mod core;
mod feedback;
mod maps;
mod utils;

const MAX_JSON_PAYLOAD: usize = 1024 * 1024; // 1 MB

#[derive(StructOpt, Debug)]
#[structopt(name = "server")]
pub struct Opt {
    // Feedback
    /// GitHub personal access token
    #[structopt(short = "t", long)]
    github_token: Option<String>,
}

#[get("/api/source_code")]
async fn source_code_handler() -> HttpResponse {
    let gh_base = "https://github.com/TUM-Dev/navigatum".to_string();
    let commit_hash = std::env::var("GIT_COMMIT_SHA");
    let github_link = match commit_hash {
        Ok(hash) => format!("{}/tree/{}", gh_base, hash),
        Err(_) => gh_base,
    };
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(github_link)
}

#[get("/api/health")]
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
        opt.github_token = match std::env::var("GITHUB_TOKEN") {
            Ok(token) => Some(token),
            Err(_) => None,
        };
    }

    let state_feedback = web::Data::new(feedback::init_state(opt));

    HttpServer::new(move || {
        // in local development we serve our website from two diverent CORS sources
        // since we need the lang cookie for api localisation, we have to add
        // Access-Control-Allow-Credentials=true to the response header
        // since origin cannot be '*' in this case, we explicitly set it
        let base_cors = match std::env::var("GIT_COMMIT_SHA") {
            Ok(_) => Cors::default().allow_any_origin(),
            Err(_) => {
                warn!("Running in local development mode. Only allowing http://localhost:8000 as origin");
                Cors::default()
                    .supports_credentials()
                    .allowed_origin("http://localhost:8000")
            }
        };
        let cors = base_cors
            .allow_any_header()
            .allowed_methods(vec!["GET", "POST"])
            .max_age(3600);

        let json_config = web::JsonConfig::default().limit(MAX_JSON_PAYLOAD);

        let logger = middleware::Logger::default().exclude("/api/health");

        App::new()
            .wrap(cors)
            .wrap(logger)
            .wrap(middleware::Compress::default())
            .app_data(json_config)
            .service(source_code_handler)
            .service(health_handler)
            .service(
                web::scope("/api/feedback")
                    .configure(feedback::configure)
                    .app_data(state_feedback.clone()),
            )
            .service(web::scope("/api").configure(core::configure))
            .service(web::scope("/maps").configure(maps::configure))
    })
    .bind(std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string()))?
    .run()
    .await
}
