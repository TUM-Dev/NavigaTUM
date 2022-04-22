use actix_cors::Cors;
use actix_web::{get, http, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use rusqlite::{params, Connection};
use structopt::StructOpt;

mod feedback;
mod search;

const MAX_JSON_PAYLOAD: usize = 1024 * 1024; // 1 MB

#[derive(StructOpt, Debug)]
#[structopt(name = "server")]
pub struct Opt {
    // Feedback
    /// GitHub personal access token
    #[structopt(short = "t", long)]
    github_token: Option<String>,
}

#[get("/api/get/{id}")]
async fn get_handler(params: web::Path<String>) -> Result<HttpResponse> {
    let id = params.into_inner();
    let conn = Connection::open("data/api_data.db").expect("Cannot open database");
    let mut stmt = conn
        .prepare("SELECT value FROM api_data WHERE key = ?")
        .expect("Cannot prepare statement");
    let result = stmt.query_row(params![id], |row| {
        let data: String = row.get_unwrap(0);
        return Ok(data);
    });
    match result {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(data)),
        Err(_) => Ok(HttpResponse::NotFound().body("Not found")),
    }
}

#[get("/api/search/{q}")]
async fn search_handler(
    _req: HttpRequest,
    params: web::Path<String>,
    web::Query(args): web::Query<search::SearchQueryArgs>,
) -> Result<HttpResponse> {
    let q = params.into_inner();
    let search_results = search::do_benchmarked_search(q, args).await?;
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

#[get("/api/health")]
async fn health_handler() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("Healthy!"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let mut opt = Opt::from_args();
    if opt.github_token.is_none() {
        let github_token = std::env::var("GITHUB_TOKEN");
        if github_token.is_ok() {
            opt.github_token = Some(github_token.unwrap());
        }
    }

    let state_feedback = web::Data::new(feedback::init_state(opt));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
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
