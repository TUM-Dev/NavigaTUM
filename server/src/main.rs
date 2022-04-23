use actix_cors::Cors;
use actix_web::{get, middleware, web, App, HttpRequest, HttpResponse, HttpServer};
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
async fn get_handler(params: web::Path<String>) -> HttpResponse {
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
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Not found"),
    }
}

#[get("/api/search/{q}")]
async fn search_handler(
    _req: HttpRequest,
    params: web::Path<String>,
    web::Query(args): web::Query<search::SearchQueryArgs>,
) -> HttpResponse {
    let q = params.into_inner();
    let search_results = search::do_benchmarked_search(q, args).await;
    HttpResponse::Ok().json(search_results)
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
