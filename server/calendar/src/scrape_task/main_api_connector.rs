use log::error;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;

fn api_url_from_env() -> Option<String> {
    let main_api_addr = std::env::var("CDN_SVC_SERVICE_HOST").ok()?;
    let main_api_port = std::env::var("CDN_SVC_SERVICE_PORT_HTTP").ok()?;

    Some(format!(
        "http://{main_api_addr}:{main_api_port}/cdn/api_data.json"
    ))
}

#[derive(Deserialize, Debug)]
pub struct ReducedRoom {
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
    fn from((key, room): (String, ReducedRoom)) -> Option<Room> {
        let url = room.props.calendar_url?;
        let regex = Regex::new(r".*cOrg=(?P<org>\d+)&cRes=(?P<cal>\d+)\D.*").unwrap();
        let captures = regex.captures(&url)?;
        Some(Room {
            sap_id: key,
            tumonline_org_id: captures.name("org")?.as_str().parse().ok()?,
            tumonline_calendar_id: captures.name("cal")?.as_str().parse().ok()?,
            tumonline_room_id: room.props.tumonline_room_nr?,
        })
    }
}

pub async fn get_all_ids() -> Vec<Room> {
    let url =
        api_url_from_env().unwrap_or_else(|| "https://nav.tum.de/cdn/api_data.json".to_string());
    let res = reqwest::get(&url).await;
    let rooms = match res {
        Ok(res) => res.json::<HashMap<String, ReducedRoom>>().await,
        Err(e) => {
            error!("Failed to contact main-api at {url}: {e:#?}");
            return vec![];
        }
    };
    match rooms {
        Ok(rooms) => rooms.into_iter().flat_map(Room::from).collect(),
        Err(e) => panic!("Failed to parse main-api response: {e:#?}"),
    }
}
