#[macro_use]
extern crate lazy_static;
use actix_web::{get, http, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use serde_json;
use std::collections::HashMap;
use std::fs;

use actix_web::client::Client;

use actix_cors::Cors;

lazy_static! {
    static ref JSON_DATA: serde_json::map::Map<String, serde_json::Value> = {
        let data = fs::read_to_string("data/api_data.json").expect("Cannot open data file");
        serde_json::from_str(&data).expect("Could not parse JSON file")
    };
}

#[get("/get/{id}")]
async fn get_data(web::Path(id): web::Path<String>) -> Result<HttpResponse> {
    if JSON_DATA.contains_key(&id) {
        Ok(HttpResponse::Ok().json(JSON_DATA.get(&id).unwrap()))
    } else {
        Ok(HttpResponse::NotFound().body("Not found".to_string()))
    }
}

#[get("/search/{q}")]
async fn search(
    _req: HttpRequest,
    web::Path(q): web::Path<String>,
    client: web::Data<Client>,
) -> Result<HttpResponse> {
    let mut post_data = HashMap::new();
    post_data.insert("q", q);

    let resp_bytes = client
        .post("http://localhost:7700/indexes/obj/search")
        .send_json(&post_data)
        .await
        .unwrap()
        .body()
        .await?;

    let resp = std::str::from_utf8(resp_bytes.as_ref()).unwrap();

    Ok(HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(resp.to_string()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    JSON_DATA.contains_key("");

    HttpServer::new(|| {
        let cors = Cors::default().allow_any_origin();

        App::new()
            .wrap(cors)
            .wrap(middleware::Compress::default())
            .data(Client::new())
            .service(get_data)
            .service(search)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
