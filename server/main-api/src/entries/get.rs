use crate::models::DBRoomKeyAlias;
use crate::utils;
use actix_web::{get, web, HttpResponse};
use log::error;
use sqlx::SqlitePool;

#[get("/api/get/{id}")]
pub async fn get_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<utils::LangQueryArgs>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let (probable_id, redirect_url) =
        match get_alias_and_redirect(&data.db, &params.into_inner()).await {
            Some(alias_and_redirect) => alias_and_redirect,
            None => return HttpResponse::NotFound().body("Not found"),
        };
    let result = match args.should_use_english() {
        true => {
            sqlx::query_scalar!("SELECT data FROM en WHERE key = ?", probable_id)
                .fetch_optional(&data.db)
                .await
        }
        false => {
            sqlx::query_scalar!("SELECT data FROM de WHERE key = ?", probable_id)
                .fetch_optional(&data.db)
                .await
        }
    };
    match result {
        Ok(d) => match d {
            None => HttpResponse::NotFound().body("Not found"),
            Some(d) => {
                let mut response_json = d.clone();
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

async fn get_alias_and_redirect(conn: &SqlitePool, query: &str) -> Option<(String, String)> {
    let result = sqlx::query_as!(
        DBRoomKeyAlias,
        r#"
        SELECT key, visible_id, type
        FROM aliases
        WHERE key = ? OR key = ?
        "#,
        query,
        query
    )
    .fetch_all(conn)
    .await;
    match result {
        Ok(d) => {
            let redirect_url = match d.len() {
                0 => return None, // not key or alias
                1 => extract_redirect_exact_match(&d[0].r#type, &d[0].visible_id),
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
