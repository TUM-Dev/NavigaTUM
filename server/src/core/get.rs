use actix_web::{get, web, HttpRequest, HttpResponse};
use log::error;
use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DetailsQuerryArgs {
    lang: Option<String>,
}

#[get("/get/{id}")]
pub async fn get_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<DetailsQuerryArgs>,
    req: HttpRequest,
) -> HttpResponse {
    let id = params.into_inner();
    let conn = Connection::open_with_flags(
        "data/api_data.db",
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("Cannot open database");

    let en_stmt = conn.prepare_cached("SELECT en FROM api_data WHERE key = ?");
    let de_stmt = conn.prepare_cached("SELECT de FROM api_data WHERE key = ?");
    // we calculate the language from the request by checking if either the query or the cookie are set to en
    let cookie_en = req.cookie("lang").map_or(false, |c| c.value() == "en");
    let arg_en = args.lang.map_or(false, |c| c == "en");
    let stmt = match arg_en || cookie_en {
        true => en_stmt,
        false => de_stmt,
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
