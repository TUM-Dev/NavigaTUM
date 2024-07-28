use actix_web::{get, web, HttpResponse};
use sqlx::Error::RowNotFound;
use sqlx::PgPool;
use tracing::error;

use crate::models::LocationKeyAlias;
use crate::localisation;

#[get("/api/get/{id}")]
pub async fn get_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<localisation::LangQueryArgs>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let id = params
        .into_inner()
        .replace(|c: char| c.is_whitespace() || c.is_control(), "");
    let Some((probable_id, redirect_url)) = get_alias_and_redirect(&data.pool, &id).await else {
        return HttpResponse::NotFound().body("Not found");
    };
    let result = if args.should_use_english() {
        sqlx::query_scalar!("SELECT data FROM en WHERE key = $1", probable_id)
            .fetch_optional(&data.pool)
            .await
    } else {
        sqlx::query_scalar!("SELECT data FROM de WHERE key = $1", probable_id)
            .fetch_optional(&data.pool)
            .await
    };
    match result {
        Ok(d) => match d {
            None => HttpResponse::NotFound().body("Not found"),
            Some(d) => {
                let mut response_json = serde_json::to_string(&d).unwrap();
                // We don't want to serialise this data at any point in the server.
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

#[tracing::instrument(skip(pool))]
async fn get_alias_and_redirect(pool: &PgPool, query: &str) -> Option<(String, String)> {
    let result = sqlx::query_as!(
        LocationKeyAlias,
        r#"
        SELECT DISTINCT key, visible_id, type
        FROM aliases
        WHERE alias = $1 OR key = $1 "#,
        query
    )
    .fetch_all(pool)
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
        Err(RowNotFound) => None,
        Err(e) => {
            error!("Error requesting alias for {query}: {e:?}");
            None
        }
    }
}

fn extract_redirect_exact_match(type_: &str, key: &str) -> String {
    match type_ {
        "campus" => format!("/campus/{key}"),
        "site" | "area" => format!("/site/{key}"),
        "building" | "joined_building" => format!("/building/{key}"),
        "room" | "virtual_room" => format!("/room/{key}"),
        "poi" => format!("/poi/{key}"),
        _ => format!("/view/{key}"), // can be triggered if we add a type but don't add it here
    }
}
