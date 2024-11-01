use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{get, web, HttpResponse};
use sqlx::Error::RowNotFound;
use sqlx::PgPool;
use tracing::error;

use crate::localisation;
use crate::models::LocationKeyAlias;

#[get("/{id}")]
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
                    .insert_header(CacheControl(vec![
                        CacheDirective::MaxAge(24 * 60 * 60), // valid for 1d
                        CacheDirective::Public,
                    ]))
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::task::LocalSet;
    use tracing::info;

    use super::*;
    use crate::{setup::tests::PostgresTestContainer, AppData};

    /// Allows tesing if a modification has changed the output of the details API
    ///
    /// The testcase can be executed via running the following command on main
    /// ```bash
    /// INSTA_OUTPUT=none INSTA_UPDATE=always DATABASE_URL=postgres://postgres:CHANGE_ME@localhost:5432 cargo test -p navigatum-server test_get_handler_unchanged -- --nocapture --include-ignored
    /// ```
    ///
    /// And then running this command on the change
    /// ```bash
    /// DATABASE_URL=postgres://postgres:CHANGE_ME@localhost:5432 cargo insta --review -- -p navigatum-server test_get_handler_unchanged --nocapture --include-ignored
    /// ```
    ///
    /// This is a ..bit.. slow, due to using a [`tokio::task::LocalSet`].
    /// Using multiple cores for this might be possible, but optimising this testcase from 10m is currently not worth it
    #[ignore]
    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_get_handler_unchanged() {
        // setup + load data into postgis
        let pg = PostgresTestContainer::new().await;
        for i in 0..20 {
            let res = crate::setup::database::load_data(&pg.pool).await;
            if let Err(e) = res {
                error!("failed to load db because {e:?}. Retrying for 20s");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            } else {
                info!("successfully initalised the db in try {i}");
                break;
            }
        }

        let keys: Vec<String> = sqlx::query_scalar!("SELECT key FROM de")
            .fetch_all(&pg.pool)
            .await
            .unwrap();
        let all_keys_len = keys.len();
        let mut resolved_cnt = 0_usize;

        for key_chunk in keys.chunks(1000) {
            let tasks = LocalSet::new();
            for key in key_chunk {
                let inner_key = key.clone();
                let inner_pool = pg.pool.clone();
                tasks.spawn_local(async move {
                    check_snapshot(inner_key, inner_pool).await;
                });
            }
            tasks.await;
            resolved_cnt += key_chunk.len();
            info!(
                "processed {resolved_cnt}/{all_keys_len} <=> {percentage}%",
                percentage = 100_f32 * (resolved_cnt as f32) / (all_keys_len as f32)
            )
        }
    }

    async fn check_snapshot(key: String, pool: PgPool) {
        let data = AppData {
            pool,
            meilisearch_initialised: Arc::new(Default::default()),
        };
        let app = actix_web::App::new()
            .app_data(web::Data::new(data))
            .service(get_handler);
        let app = actix_web::test::init_service(app).await;
        let req = actix_web::test::TestRequest::get()
            .uri(&format!("/{key}"))
            .to_request();
        let (_, resp) = actix_web::test::call_service(&app, req).await.into_parts();

        assert_eq!(resp.status().as_u16(), 200);

        let body_box = resp.into_body();
        let body_bytes = actix_web::body::to_bytes(body_box).await.unwrap();
        let body_str = String::from_utf8(body_bytes.into_iter().collect()).unwrap();
        let body_value: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        let mut settings = insta::Settings::clone_current();
        settings.set_sort_maps(true);
        settings.set_snapshot_path("location_get_handler");
        settings.bind(|| {
            insta::assert_json_snapshot!(key.clone(), body_value);
        });
    }
}
