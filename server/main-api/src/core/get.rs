use crate::utils;
use actix_web::{get, web, HttpResponse};
use diesel::prelude::*;
use log::error;

#[get("/get/{id}")]
pub async fn get_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<utils::LangQueryArgs>,
) -> HttpResponse {
    let id = params.into_inner();
    let conn = &mut utils::establish_connection();
    let result = match args.should_use_english() {
        true => {
            use crate::schema::en::dsl::*;
            en.filter(key.eq(&id)).select(data).load::<String>(conn)
        }
        false => {
            use crate::schema::de::dsl::*;
            de.filter(key.eq(&id)).select(data).load::<String>(conn)
        }
    };
    match result {
        Ok(d) => match d.len() {
            0 => HttpResponse::NotFound().body("Not found"),
            _ => HttpResponse::Ok()
                .content_type("application/json")
                .body(d[0].clone()),
        },
        Err(e) => {
            error!("Error requesting details for {id}: {e:?}");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Internal Server Error")
        }
    }
}
