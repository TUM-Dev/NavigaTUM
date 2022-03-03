#[macro_use]
extern crate lazy_static;

use std::fs;

use actix_cors::Cors;
use actix_web::{get, http, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use structopt::StructOpt;

mod feedback;
mod search;

const MAX_JSON_PAYLOAD: usize = 1024 * 1024; // 1 MB

#[derive(StructOpt, Debug)]
#[structopt(name = "server")]
pub struct Opt {
    // Feedback
    /// GitLab instance domain
    #[structopt(short = "g", long)]
    gitlab_domain: Option<String>,

    /// GitLab access token
    #[structopt(short = "t", long)]
    gitlab_token: Option<String>,

    /// GitLab feedback project id
    #[structopt(short = "f", long)]
    feedback_project: Option<i32>,
}

lazy_static! {
    static ref JSON_DATA: serde_json::map::Map<String, serde_json::Value> = {
        let data = fs::read_to_string("data/api_data.json")
            .expect("Cannot open data file. (not found at 'data/api_data.json')");
        serde_json::from_str(&data).expect("Could not parse JSON file")
    };
}

#[get("/api/get/{id}")]
async fn get_handler(params: web::Path<String>) -> Result<HttpResponse> {
    let id = params.into_inner();
    if JSON_DATA.contains_key(&id) {
        Ok(HttpResponse::Ok().json(JSON_DATA.get(&id).unwrap()))
    } else {
        Ok(HttpResponse::NotFound().body("Not found".to_string()))
    }
}

#[get("/api/search/{q}")]
async fn search_handler(
    _req: HttpRequest,
    params: web::Path<String>,
    web::Query(args): web::Query<search::SearchQueryArgs>,
) -> Result<HttpResponse> {
    let q = params.into_inner();
    let search_results = search::do_search(q, args).await?;
    let result_json = serde_json::to_string(&search_results)?;

    Ok(HttpResponse::Ok()
        .insert_header((http::header::CONTENT_TYPE, "application/json"))
        .body(result_json))
}

#[get("/api/source_code")]
async fn source_code_handler() -> Result<HttpResponse> {
    let gh_base = "https://github.com/TUM-Dev/navigatum".to_string();
    let commit_hash = std::env::var("GIT_COMMIT_SHA");
    if commit_hash.is_ok() {
        let github_link = format!("{}{}{}", gh_base, "/tree/", commit_hash.unwrap());
        Ok(HttpResponse::Ok().body(github_link))
    } else {
        Ok(HttpResponse::Ok().body(gh_base))
    }
}

#[get("/health")]
async fn health_handler() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("Healthy!"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let opt = Opt::from_args();

    // This causes lazy_static to evaluate
    JSON_DATA.contains_key("");

    let state_feedback = web::Data::new(feedback::init_state(opt));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allowed_methods(vec!["GET", "POST"])
            .max_age(3600);

        let json_config = web::JsonConfig::default().limit(MAX_JSON_PAYLOAD);

        let logger = middleware::Logger::default();

        App::new()
            .wrap(cors)
            .wrap(logger)
            .wrap(middleware::Compress::default())
            .app_data(json_config)
            .service(get_handler)
            .service(search_handler)
            .service(source_code_handler)
            .service(health_handler)
            .service(
                web::scope("/api/feedback")
                    .configure(feedback::configure)
                    .app_data(state_feedback.clone()),
            )
    })
    .bind(std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string()))?
    .run()
    .await
}
