use crate::models::DBRoomKeyAlias;
use crate::utils;
use actix_web::{get, web, HttpResponse};
use diesel::prelude::*;
use log::error;

#[get("/api/get/{id}")]
pub async fn get_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<utils::LangQueryArgs>,
) -> HttpResponse {
    let conn = &mut utils::establish_connection();
    let (probable_id, redirect_url) = match get_alias_and_redirect(conn, &params.into_inner()) {
        Some(alias_and_redirect) => alias_and_redirect,
        None => return HttpResponse::NotFound().body("Not found"),
    };
    let result = match args.should_use_english() {
        true => {
            use crate::schema::en::dsl;
            dsl::en
                .filter(dsl::key.eq(&probable_id))
                .select(dsl::data)
                .load::<String>(conn)
        }
        false => {
            use crate::schema::de::dsl;
            dsl::de
                .filter(dsl::key.eq(&probable_id))
                .select(dsl::data)
                .load::<String>(conn)
        }
    };
    match result {
        Ok(d) => match d.len() {
            0 => HttpResponse::NotFound().body("Not found"),
            _ => {
                let mut response_json = d[0].clone();
                // We don not want to serialise this data at any point in the server.
                // This just flows through the server, but adding redirect_url to the response is necessary
                response_json.pop(); // remove last }
                response_json.push_str(&format!(",\"redirect_url\":\"{redirect_url}\"}}",));
                HttpResponse::Ok()
                    .content_type("application/json")
                    .body(response_json)
            }
        },
        Err(e) => {
            error!("Error requesting details for {probable_id}: {e:?}");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Internal Server Error")
        }
    }
}

fn get_alias_and_redirect(conn: &mut SqliteConnection, query: &str) -> Option<(String, String)> {
    use crate::schema::aliases::dsl::{alias, aliases, key, type_, visible_id};
    let result = aliases
        .filter(alias.eq(query).or(key.eq(query)))
        .select((key, visible_id, type_))
        .distinct()
        .load::<DBRoomKeyAlias>(conn);
    match result {
        Ok(d) => {
            let redirect_url = match d.len() {
                0 => return None, // not key or alias
                1 => extract_redirect_exact_match(&d[0].type_, &d[0].visible_id),
                _ => {
                    let keys = d
                        .clone()
                        .into_iter()
                        .map(|a| a.key)
                        .collect::<Vec<String>>();
                    format!("/search?q={}", keys.join("+"))
                }
            };
            Some((d[0].key.clone(), redirect_url))
        }
        Err(e) => {
            error!("Error requesting alias for {query}: {e:?}");
            None
        }
    }
}

fn extract_redirect_exact_match(type_: &str, key: &str) -> String {
    match type_ {
        "root" => String::new(),
        "campus" => format!("/campus/{key}"),
        "site" | "area" => format!("/site/{key}"),
        "building" | "joined_building" => format!("/building/{key}"),
        "room" | "virtual_room" => format!("/room/{key}"),
        "poi" => format!("/poi/{key}"),
        _ => format!("/view/{key}"), // can be triggered if we add a type but don't add it here
    }
}
