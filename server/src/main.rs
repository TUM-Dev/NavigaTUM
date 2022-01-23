#[macro_use]
extern crate lazy_static;
use std::fs;

use actix_cors::Cors;
use actix_web::{get, http, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use structopt::StructOpt;

mod search;
mod feedback;


const MAX_JSON_PAYLOAD: usize = 1024 * 1024;  // 1 MB

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
        let data = fs::read_to_string("data/api_data.json").expect("Cannot open data file");
        serde_json::from_str(&data).expect("Could not parse JSON file")
    };
}

#[get("/get/{id}")]
async fn get_handler(web::Path(id): web::Path<String>) -> Result<HttpResponse> {
    if JSON_DATA.contains_key(&id) {
        Ok(HttpResponse::Ok().json(JSON_DATA.get(&id).unwrap()))
    } else {
        Ok(HttpResponse::NotFound().body("Not found".to_string()))
    }
}

#[get("/search/{q}")]
async fn search_handler(
    _req: HttpRequest,
    web::Path(q): web::Path<String>,
) -> Result<HttpResponse> {
    let search_results = search::do_search(q).await?;
    let result_json = serde_json::to_string(&search_results)?;

    Ok(HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(result_json))
}

#[get("/source_code")]
async fn source_code_handler() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("https://git.fs.tum.de/navigatum/navigatum-server".to_string()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let opt = Opt::from_args();

    // This causes lazy_static to evaluate
    JSON_DATA.contains_key("");

    let state_feedback = web::Data::new(feedback::init_state(opt));

    HttpServer::new(move || {
        let cors = Cors::default().allowed_origin("http://localhost:8080")
                                  .allowed_origin("new.roomfinder.tum.sexy")
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
            .service(web::scope("/feedback").configure(feedback::configure)
                                            .app_data(state_feedback.clone()))
    })
    .bind(std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string()))?
    .run()
    .await
}
