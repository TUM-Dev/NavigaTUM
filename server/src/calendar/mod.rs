use std::collections::HashMap;

use actix_web::{post, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::error;

use crate::calendar::models::{CalendarLocation, Event, LocationEvents};
use crate::limited::hash_map::LimitedHashMap;
use crate::limited::vec::LimitedVec;

mod connectum;
mod models;
pub mod refresh;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Arguments {
    ids: Vec<String>,
    /// eg. 2039-01-19T03:14:07+1
    start_after: DateTime<Utc>,
    /// eg. 2042-01-07T00:00:00 UTC
    end_before: DateTime<Utc>,
}

impl Arguments {
    fn validate_ids(&self) -> Result<Vec<String>, HttpResponse> {
        let ids = self
            .ids
            .clone()
            .into_iter()
            .map(|s| s.replace(|c: char| c.is_whitespace() || c.is_control(), ""))
            .collect::<Vec<String>>();
        if ids.len() > 10 {
            return Err(HttpResponse::BadRequest().body("Too many ids to query. We suspect that users don't need this. If you need this limit increased, please send us a message"));
        };
        if ids.is_empty() {
            return Err(HttpResponse::BadRequest().body("No id requested"));
        };
        Ok(ids)
    }
}

#[post("/api/calendar")]
pub async fn calendar_handler(
    web::Json(args): web::Json<Arguments>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let ids = match args.validate_ids() {
        Ok(ids) => ids,
        Err(e) => return e,
    };
    let locations = match get_locations(&data.pool, &ids).await {
        Ok(l) => l.0,
        Err(e) => return e,
    };
    if let Err(e) = validate_locations(&ids, &locations) {
        return e;
    }
    match get_from_db(&data.pool, &locations, &args.start_after, &args.end_before).await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => {
            error!("could not get entries from the db for {ids:?} because {e:?}");
            HttpResponse::InternalServerError()
                .body("could not get calendar entries, please try again later")
        }
    }
}

fn validate_locations(ids: &[String], locations: &[CalendarLocation]) -> Result<(), HttpResponse> {
    for id in ids {
        if !locations.iter().any(|l| &l.key == id) {
            return Err(HttpResponse::BadRequest().body("Requested id {id} does not exist"));
        }
    }
    assert_eq!(locations.len(), ids.len());
    for loc in locations {
        if loc.last_calendar_scrape_at.is_none() {
            return Err(HttpResponse::ServiceUnavailable().body(format!("Room {key}/{url:?} calendar entry is currently in the process of being scraped, please try again later", key = loc.key, url = loc.calendar_url)));
        };
    }
    for loc in locations {
        if loc.calendar_url.is_none() {
            return Err(HttpResponse::NotFound()
                .content_type("text/plain")
                .body(format!(
                    "Room {key}/{url:?} does not have a calendar",
                    key = loc.key,
                    url = loc.calendar_url
                )));
        };
    }
    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn get_locations(
    pool: &PgPool,
    ids: &[String],
) -> Result<LimitedVec<CalendarLocation>, HttpResponse> {
    match sqlx::query_as!(CalendarLocation, "SELECT key,name,last_calendar_scrape_at,calendar_url,type,type_common_name FROM de WHERE key = ANY($1::text[])", ids).fetch_all(pool).await {
        Err(e) => {
            error!("could not refetch due to {e:?}");
            Err(HttpResponse::InternalServerError().body("could not get calendar entries, please try again later"))
        }
        Ok(locations) => Ok(LimitedVec(locations)),
    }
}

#[tracing::instrument(skip(pool),ret(level = tracing::Level::TRACE))]
async fn get_from_db(
    pool: &PgPool,
    locations: &[CalendarLocation],
    start_after: &DateTime<Utc>,
    end_before: &DateTime<Utc>,
) -> anyhow::Result<LimitedHashMap<String, LocationEvents>> {
    let mut located_events: HashMap<String, LocationEvents> = HashMap::new();
    for location in locations {
        let events = sqlx::query_as!(Event, r#"SELECT id,room_code,start_at,end_at,title_de,title_en,stp_type,entry_type,detailed_entry_type
            FROM calendar
            WHERE room_code = $1 AND start_at >= $2 AND end_at <= $3"#,
            location.key, start_after, end_before).fetch_all(pool).await?;
        located_events.insert(
            location.key.clone(),
            LocationEvents {
                location: location.clone(),
                events,
            },
        );
    }
    Ok(LimitedHashMap(located_events))
}

#[cfg(all(feature = "test-with-geodata", test))]
mod db_tests {
    use std::sync::Arc;

    use actix_web::http::header::ContentType;
    use actix_web::test;
    use actix_web::App;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use pretty_assertions::assert_eq;
    use serde_json::Value;

    use crate::setup::tests::PostgresTestContainer;
    use crate::AppData;

    use super::*;

    /// Workaround because [`Option::unwrap()`] is not (yet) available in const context.
    /// See https://github.com/rust-lang/rust/issues/67441 for further context
    const fn unwrap<T: Copy>(opt: Option<T>) -> T {
        match opt {
            Some(val) => val,
            None => panic!("unwrapped None"),
        }
    }
    const fn datetime_from_ymd(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        let date = unwrap(NaiveDate::from_ymd_opt(year, month, day));
        let time = unwrap(NaiveTime::from_num_seconds_from_midnight_opt(0, 0));
        let naive_datetime = NaiveDateTime::new(date, time);
        DateTime::from_naive_utc_and_offset(naive_datetime, Utc)
    }
    const TIME_Y2K: DateTime<Utc> = datetime_from_ymd(2000, 1, 1);
    const TIME_2010: DateTime<Utc> = datetime_from_ymd(2010, 1, 1);
    const TIME_2012: DateTime<Utc> = datetime_from_ymd(2012, 1, 1);
    const TIME_2014: DateTime<Utc> = datetime_from_ymd(2014, 1, 1);
    const TIME_2016: DateTime<Utc> = datetime_from_ymd(2016, 1, 1);
    const TIME_2018: DateTime<Utc> = datetime_from_ymd(2018, 1, 1);
    const TIME_2020: DateTime<Utc> = datetime_from_ymd(2020, 1, 1);

    fn sample_data() -> (Vec<(String, Value)>, Vec<Event>) {
        (
            vec![
                (
                    "5121.EG.003".into(),
                    serde_json::json!({"aliases":["003@5121"],"coords":{"accuracy":"building","lat":48.26842603718826,"lon":11.677995005953209,"source":"inferred"},"id":"5121.EG.003","maps":{"default":"interactive"},"name":"5121.EG.003 (Computerraum)","parent_names":["Standorte","Garching Forschungszentrum","Physik","Maier-Leibnitz-Laboratorium (MLL), TUM & LMU","Atlashalle"],"parents":["root","garching","physik","mll","5121"],"poi":{"nearby_public_transport":{"mvg":[]}},"props":{"calendar_url":"https://campus.tum.de/3","computed":[{"name":"Raumkennung","text":"5121.EG.003"},{"name":"Architekten-Name","text":"003"},{"name":"Stockwerk","text":"Erdgeschoss"},{"name":"Adresse","text":"Am Coulombwall 6, 85748 Garching b. München"}],"operator":{"code":"TUPELMU","id":39536,"name":"Ludwig-Maximilians-Universität München (LMU)","url":"https://campus.tum.de/tumonline/webnav.navigate_to?corg=39536"},"tumonline_room_nr":45064},"ranking_factors":{"rank_combined":10,"rank_type":100,"rank_usage":10},"sources":{"base":[{"name":"TUMonline","url":"https://campus.tum.de/tumonline/ee/ui/ca2/app/desktop/#/pl/ui/$ctx/45064"}]},"type":"room","type_common_name":"Serverraum","usage":{"din_277":"TF8.9","din_277_desc":"Sonstige betriebstechnische Anlagen","name":"Serverraum"},"redirect_url":"/room/5121.EG.003"}),
                ),
                (
                    "5121.EG.002".into(),
                    serde_json::json!({"aliases":["002@5121"],"coords":{"accuracy":"building","lat":48.26842603718826,"lon":11.677995005953209,"source":"inferred"},"id":"5121.EG.002","maps":{"default":"interactive"},"name":"5121.EG.002 (Testroom)","parent_names":["Standorte","Garching Forschungszentrum","Physik","Maier-Leibnitz-Laboratorium (MLL),TUM & LMU","Atlashalle"],"parents":["root","garching","physik","mll","5121"],"poi":{"nearby_public_transport":{"mvg":[]}},"props":{"computed":[{"name":"Raumkennung","text":"5121.EG.002"},{"name":"Architekten-Name","text":"002"},{"name":"Stockwerk","text":"Erdgeschoss"},{"name":"Adresse","text":"Am Coulombwall 6,85748 Garching b. München"}  ],"operator":{"code":"TUPELMU","id":39536,"name":"Ludwig-Maximilians-Universität München (LMU)","url":"https://campus.tum.de/tumonline/webnav.navigate_to?corg=39536"},"tumonline_room_nr":44904},"ranking_factors":{"rank_combined":10,"rank_type":100,"rank_usage":10},"sources":{"base":[{"name":"TUMonline","url":"https://campus.tum.de/tumonline/ee/ui/ca2/app/desktop/#/pl/ui/$ctx/44904"}  ]},"type":"room","type_common_name":"Versuchshalle","usage":{"din_277":"NF3.3","din_277_desc":"Technologische Labors","name":"Versuchshalle"},"redirect_url":"/room/5121.EG.002"}),
                ),
                (
                    "5121.EG.001".into(),
                    serde_json::json!({"aliases":["001@5121"],"coords":{"accuracy":"building","lat":48.26842603718826,"lon":11.677995005953209,"source":"inferred"},"id":"5121.EG.001","maps":{"default":"interactive"},"name":"5121.EG.001 (Montage- und Versuchshalle)","parent_names":["Standorte","Garching Forschungszentrum","Physik","Maier-Leibnitz-Laboratorium (MLL),TUM & LMU","Atlashalle"],"parents":["root","garching","physik","mll","5121"],"poi":{"nearby_public_transport":{"mvg":[]}},"props":{"calendar_url":"https://campus.tum.de/1","computed":[{"name":"Raumkennung","text":"5121.EG.001"},{"name":"Architekten-Name","text":"001"},{"name":"Stockwerk","text":"Erdgeschoss"},{"name":"Adresse","text":"Am Coulombwall 6,85748 Garching b. München"}  ],"operator":{"code":"TUPELMU","id":39536,"name":"Ludwig-Maximilians-Universität München (LMU)","url":"https://campus.tum.de/tumonline/webnav.navigate_to?corg=39536"},"tumonline_room_nr":44904},"ranking_factors":{"rank_combined":10,"rank_type":100,"rank_usage":10},"sources":{"base":[{"name":"TUMonline","url":"https://campus.tum.de/tumonline/ee/ui/ca2/app/desktop/#/pl/ui/$ctx/44904"}  ]},"type":"room","type_common_name":"Versuchshalle","usage":{"din_277":"NF3.3","din_277_desc":"Technologische Labors","name":"Versuchshalle"},"redirect_url":"/room/5121.EG.001"}),
                ),
            ],
            vec![
                Event {
                    id: 1,
                    room_code: "5121.EG.003".into(),
                    start_at: TIME_2012.clone(),
                    end_at: TIME_2014.clone(),
                    title_de: "Quantenteleportation".into(),
                    title_en: "Quantum teleportation".into(),
                    stp_type: Some("Vorlesung mit Zentralübung".into()),
                    entry_type: models::EventType::Lecture.to_string(),
                    detailed_entry_type: "Abhaltung".into(),
                },
                Event {
                    id: 2,
                    room_code: "5121.EG.003".into(),
                    start_at: TIME_2014.clone(),
                    end_at: TIME_2016.clone(),
                    title_de: "Quantenteleportation 2".into(),
                    title_en: "Quantum teleportation 2".into(),
                    stp_type: Some("Vorlesung mit Zentralübung".into()),
                    entry_type: models::EventType::Lecture.to_string(),
                    detailed_entry_type: "Abhaltung".into(),
                },
                Event {
                    id: 3,
                    room_code: "5121.EG.001".into(),
                    start_at: TIME_2014.clone(),
                    end_at: TIME_2016.clone(),
                    title_de: "Wartung".into(),
                    title_en: "maintenance".into(),
                    stp_type: Some("Vorlesung mit Zentralübung".into()),
                    entry_type: models::EventType::Barred.to_string(),
                    detailed_entry_type: "Abhaltung".into(),
                },
                Event {
                    id: 4,
                    room_code: "5121.EG.001".into(),
                    start_at: TIME_Y2K.clone(),
                    end_at: TIME_2020.clone(),
                    title_de: "Quantenteleportation 3".into(),
                    title_en: "Quantum teleportation 3".into(),
                    stp_type: Some("Vorlesung".into()),
                    entry_type: models::EventType::Other.to_string(),
                    detailed_entry_type: "Abhaltung".into(),
                },
                Event {
                    id: 5,
                    room_code: "5121.EG.001".into(),
                    start_at: TIME_Y2K.clone(),
                    end_at: TIME_2010.clone(),
                    title_de: "Quantenteleportation 3".into(),
                    title_en: "Quantum teleportation 3".into(),
                    stp_type: Some("Vorlesung".into()),
                    entry_type: models::EventType::Exam.to_string(),
                    detailed_entry_type: "Abhaltung".into(),
                },
            ],
        )
    }

    async fn load_sample_data(pool: &PgPool, now_rfc3339: &str) {
        let mut tx = pool.begin().await.unwrap();
        let (locations, events) = sample_data();
        for (key, data) in locations {
            for lang in ["de", "en"] {
                let query = format!("INSERT INTO {lang}(key,data,last_calendar_scrape_at) VALUES ('{key}','{data}','{now_rfc3339}')");
                sqlx::query(&query).execute(&mut *tx).await.unwrap();
            }
        }

        for event in events {
            event.store(&mut tx).await.unwrap();
        }
        tx.commit().await.unwrap();
    }

    #[actix_web::test]
    async fn test_index_get() {
        // setup + load data into postgis
        let pg = PostgresTestContainer::new().await;
        let now = Utc::now();
        let now = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true); // throwing away accuracy for simpler testing
        load_sample_data(&pg.pool, &now).await;
        // set up the http service/api/calendar
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    pool: pg.pool.clone(),
                    meilisearch_initialised: Arc::new(Default::default()),
                }))
                .service(calendar_handler),
        )
        .await;
        // -- send requests and assert response --
        {
            // missing required query parameters
            let req = test::TestRequest::post()
                .uri("/api/calendar")
                .insert_header(ContentType::json())
                .to_request();
            let (_, resp) = test::call_service(&app, req).await.into_parts();

            let (status, actual) = run_testcase(resp).await;
            assert_eq!(status, 400);
            insta::assert_snapshot!(actual, @r###""Json deserialize error: EOF while parsing a value at line 1 column 0""###);
        }
        {
            // missing required query parameters
            let args = Arguments {
                end_before: Utc::now(),
                start_after: Utc::now(),
                ids: vec![],
            };
            let req = test::TestRequest::post()
                .uri("/api/calendar")
                .set_json(args)
                .insert_header(ContentType::json())
                .to_request();
            let (_, resp) = test::call_service(&app, req).await.into_parts();

            let (status, actual) = run_testcase(resp).await;
            assert_eq!(status, 400);
            insta::assert_snapshot!(actual, @r###""No id requested""###);
        }
        {
            // way too many parameters
            let args = Arguments {
                end_before: Utc::now(),
                start_after: Utc::now(),
                ids: (0..10_000).map(|i| i.to_string()).collect(),
            };
            let req = test::TestRequest::post()
                .uri("/api/calendar")
                .set_json(args)
                .insert_header(ContentType::json())
                .to_request();
            let (_, resp) = test::call_service(&app, req).await.into_parts();

            let (status, actual) = run_testcase(resp).await;
            assert_eq!(status, 400);
            insta::assert_snapshot!(actual, @r###""Too many ids to query. We suspect that users don't need this. If you need this limit increased, please send us a message""###);
        }
        {
            // room without a calendar
            let args = Arguments {
                end_before: Utc::now(),
                start_after: Utc::now(),
                ids: vec!["5121.EG.002".into()],
            };
            let req = test::TestRequest::post()
                .uri("/api/calendar")
                .set_json(args)
                .insert_header(ContentType::json())
                .to_request();
            let (_, resp) = test::call_service(&app, req).await.into_parts();

            let (status, actual) = run_testcase(resp).await;
            assert_eq!(status, 404);
            insta::assert_snapshot!(actual, @r###""Room 5121.EG.002/None does not have a calendar""###);
        }
        {
            // show all entries of 5121.EG.003
            let args = Arguments {
                start_after: TIME_Y2K.clone(),
                end_before: TIME_2020.clone(),
                ids: vec!["5121.EG.003".into()],
            };
            let req = test::TestRequest::post()
                .uri("/api/calendar")
                .set_json(args)
                .insert_header(ContentType::json())
                .to_request();
            let (_, resp) = test::call_service(&app, req).await.into_parts();

            let (status, actual) = run_testcase(resp).await;
            assert_eq!(status, 200);
            insta::assert_yaml_snapshot!(actual, {".**.last_calendar_scrape_at" => "[last_calendar_scrape_at]"});
        }
        {
            // show both rooms, but a limited timeframe
            let args = Arguments {
                start_after: *TIME_2012,
                end_before: *TIME_2014,
                ids: vec!["5121.EG.003".into(), "5121.EG.001".into()],
            };
            let req = test::TestRequest::post()
                .uri("/api/calendar")
                .set_json(args)
                .insert_header(ContentType::json())
                .to_request();
            let (_, resp) = test::call_service(&app, req).await.into_parts();

            let (status, actual) = run_testcase(resp).await;
            assert_eq!(status, 200);
            insta::assert_yaml_snapshot!(actual, {".**.last_calendar_scrape_at" => "[last_calendar_scrape_at]"});
        }
    }

    async fn run_testcase(resp: HttpResponse) -> (u16, Value) {
        let actual_status = resp.status().as_u16();
        let body_box = resp.into_body();
        let body_bytes = actix_web::body::to_bytes(body_box).await.unwrap();
        let body_text = String::from_utf8(body_bytes.into_iter().collect()).unwrap();
        // if the expected value cleanly deserializes into json, we should compare using this
        let body = if let Ok(actual) = serde_json::from_str::<Value>(&body_text) {
            actual
        } else {
            Value::String(body_text)
        };
        (actual_status, body)
    }
}
