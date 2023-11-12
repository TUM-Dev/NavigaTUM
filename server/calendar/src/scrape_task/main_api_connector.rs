use chrono::{NaiveDateTime, Utc};
use log::{error, info};
use regex::Regex;
use serde::Deserialize;
use sqlx::PgPool;

fn api_url_from_env() -> Option<String> {
    let main_api_addr = std::env::var("CDN_SVC_SERVICE_HOST").ok()?;
    let main_api_port = std::env::var("CDN_SVC_SERVICE_PORT_HTTP").ok()?;

    Some(format!(
        "http://{main_api_addr}:{main_api_port}/cdn/api_data.json"
    ))
}

#[derive(Deserialize, Debug)]
pub struct ReducedRoom {
    id: String,
    props: ReducedRoomProps,
}

#[derive(Deserialize, Debug)]
pub struct ReducedRoomProps {
    calendar_url: Option<String>, //tumonline_room_nr and calendar_url are sometimes not present, but only ever both
    tumonline_room_nr: Option<i32>,
}

#[derive(Clone, Debug, Default)]
pub struct Room {
    pub sap_id: String,
    pub tumonline_org_id: i32,
    pub tumonline_calendar_id: i32,
    pub tumonline_room_id: i32,
}

impl Room {
    fn from(room: ReducedRoom) -> Option<Room> {
        let url = room.props.calendar_url?;
        let regex = Regex::new(r".*cOrg=(?P<org>\d+)&cRes=(?P<cal>\d+)\D.*").unwrap();
        let captures = regex.captures(&url)?;
        Some(Room {
            sap_id: room.id,
            tumonline_org_id: captures.name("org")?.as_str().parse().ok()?,
            tumonline_calendar_id: captures.name("cal")?.as_str().parse().ok()?,
            tumonline_room_id: room.props.tumonline_room_nr?,
        })
    }
}

pub async fn get_all_ids(conn: &PgPool) -> Vec<Room> {
    let url =
        api_url_from_env().unwrap_or_else(|| "https://nav.tum.de/cdn/api_data.json".to_string());
    let res = reqwest::get(&url).await;
    let rooms = match res {
        Ok(res) => res.json::<Vec<ReducedRoom>>().await,
        Err(e) => {
            error!("Failed to contact main-api at {url}: {e:#?}");
            return vec![];
        }
    };
    let rooms: Vec<Room> = match rooms {
        Ok(rooms) => rooms.into_iter().filter_map(Room::from).collect(),
        Err(e) => panic!("Failed to parse main-api response: {e:#?}"),
    };
    let start_time = Utc::now().naive_utc();
    store_in_db(conn, &rooms, &start_time).await;
    delete_stale_results(conn, start_time).await;
    rooms
}

async fn store_in_db(conn: &PgPool, rooms_to_store: &[Room], start_time: &NaiveDateTime) {
    info!(
        "Storing {cnt} rooms in database",
        cnt = rooms_to_store.len()
    );
    for room in rooms_to_store {
        let room = crate::models::Room {
            key: room.sap_id.clone(),
            tumonline_org_id: room.tumonline_org_id,
            tumonline_calendar_id: room.tumonline_calendar_id,
            tumonline_room_id: room.tumonline_room_id,
            last_scrape: *start_time,
        };
        if let Err(e) =sqlx::query!(r#"
            INSERT INTO rooms(key,tumonline_org_id,tumonline_calendar_id,tumonline_room_id,last_scrape)
            VALUES ($1,$2,$3,$4,$5)
            ON CONFLICT (key) DO UPDATE SET
              tumonline_org_id=$2,
              tumonline_calendar_id=$3,
              tumonline_room_id=$4,
              last_scrape=$5"#,
            room.key,
            room.tumonline_org_id,
            room.tumonline_calendar_id,
            room.tumonline_room_id,
            room.last_scrape)
            .execute(conn)
            .await           {
                error!("Error inserting into database: {e:?}");
            }
    }
}
async fn delete_stale_results(conn: &PgPool, start_time: NaiveDateTime) {
    info!("Deleting stale rooms from the database");
    if let Err(e) = sqlx::query!("DELETE FROM rooms WHERE last_scrape < $1", start_time)
        .execute(conn)
        .await
    {
        error!("Error deleting stale rooms from database: {e:?}");
    }
}
