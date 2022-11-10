use crate::utils;
use actix_web::{get, web, HttpResponse};
use log::error;
use rusqlite::{Connection, OpenFlags};

#[get("/get/{id}")]
pub async fn get_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<utils::LangQueryArgs>,
) -> HttpResponse {
    let id = params.into_inner();
    let conn = Connection::open_with_flags(
        "data/api_data.db",
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("Cannot open database");

    let stmt = match args.should_use_english() {
        false => conn.prepare_cached("SELECT data FROM de WHERE key = ?"),
        true => conn.prepare_cached("SELECT data FROM en WHERE key = ?"),
    };
    let result = match stmt {
        Ok(mut stmt) => stmt.query_row([id], |row| {
            let data: String = row.get_unwrap(0);
            Ok(data)
        }),
        Err(e) => {
            error!("Error preparing statement: {:?}", e);
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Internal Server Error");
        }
    };
    match result {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json")
            .body(data), // .json(data) would have quoted the result. We instead want the content.
        Err(_) => HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Not found"),
    }
}
