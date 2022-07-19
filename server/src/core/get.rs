use actix_web::{get, web, HttpResponse};
use rusqlite::{Connection, OpenFlags};

#[get("/get/{id}")]
pub async fn get_handler(params: web::Path<String>) -> HttpResponse {
    let id = params.into_inner();
    let conn = Connection::open_with_flags(
        "data/api_data.db",
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("Cannot open database");
    let mut stmt = conn
        .prepare_cached("SELECT value FROM api_data WHERE key = ?")
        .expect("Cannot prepare statement");
    let result = stmt.query_row([id], |row| {
        let data: String = row.get_unwrap(0);
        Ok(data)
    });
    match result {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json")
            .body(data), // .json(data) would have quoted the result. We instead want the content.
        Err(_) => HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Not found"),
    }
}
