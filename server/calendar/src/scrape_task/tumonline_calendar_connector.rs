use crate::models::XMLEvent;
use crate::scrape_task::main_api_connector::Room;
use crate::scrape_task::scrape_room_task::ScrapeRoomTask;
use chrono::{NaiveDateTime, Utc};
use log::{debug, error, warn};
use minidom::Element;
use rand::Rng;
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

enum RequestStatus {
    Success(String),
    Timeout,
    NotFound,
    OneDayFaulty,
    Error,
}

async fn request_body(url: String) -> RequestStatus {
    let request = reqwest::get(&url).await;
    let body = match request {
        Ok(response) => match response.status().as_u16() {
            200 => response.text().await,
            404 => return RequestStatus::NotFound,
            _ => {
                error!("Error sending request (invalid status code): {response:?}");
                return RequestStatus::Error;
            }
        },
        Err(e) => {
            if e.is_timeout() {
                return RequestStatus::Timeout;
            }
            error!("Error sending request: {e:?}");
            return RequestStatus::Error;
        }
    };
    let res_string = match body {
        Ok(body) => body,
        Err(e) => {
            error!("Error converting body to string: {e:?}");
            return RequestStatus::Error;
        }
    };
    match res_string.as_str() {
        "" => RequestStatus::OneDayFaulty,
        _ => RequestStatus::Success(res_string),
    }
}

fn construct_hm(elem: &Element) -> HashMap<String, String> {
    let mut hm: HashMap<String, String> = HashMap::new();
    let attrs = elem.children().filter(|e| e.is("attribute", NS));
    let readable_attrs = attrs.map(|e| (e.attr("cor:attrID").unwrap(), e.text()));
    readable_attrs.for_each(|(attr, val)| {
        hm.insert(attr.to_string(), val);
    });

    hm
}

fn xml_event_from_hm(key: String, xml_hm: &HashMap<String, String>) -> XMLEvent {
    let other_keys = xml_hm
        .keys()
        .filter(|s| {
            !matches!(
                s.to_string().as_str(),
                "dtstart"
                        | "dtend"
                        | "dtstamp"
                        | "duration" // ignored
                        | "eventID"
                        | "eventTitle"
                        | "singleEventID"
                        | "singleEventTypeID"
                        | "singleEventIDSuccessor" // ignored
                        | "singleEventTypeName"
                        | "eventTypeID"
                        | "eventTypeName"
                        | "courseTypeName"
                        | "courseType"
                        | "courseCode"
                        | "courseSemesterHours"
                        | "groupID"
                        | "group"
                        | "statusID"
                        | "status"
                        |"comment"
            )
        })
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if !other_keys.is_empty() {
        error!("found additional key(s) in hashmap: {other_keys:?}");
    }
    XMLEvent {
        key,
        dtstart: extract_dt(xml_hm, "dtstart").unwrap(),
        dtend: extract_dt(xml_hm, "dtend").unwrap(),
        dtstamp: extract_dt(xml_hm, "dtstamp").unwrap(),
        event_id: extract_i32(xml_hm, "eventID").unwrap(),
        event_title: extract_str(xml_hm, "eventTitle")
            .unwrap_or_else(|| "Title not available".to_string()), // some deleted entries are broken in this sens
        single_event_id: extract_i32(xml_hm, "singleEventID").unwrap(),
        single_event_type_id: extract_str(xml_hm, "singleEventTypeID").unwrap(),
        single_event_type_name: extract_str(xml_hm, "singleEventTypeName").unwrap(),
        event_type_id: extract_str(xml_hm, "eventTypeID").unwrap(),
        event_type_name: extract_str(xml_hm, "eventTypeName"),
        course_type_name: extract_str(xml_hm, "courseTypeName"),
        course_type: extract_str(xml_hm, "courseType"),
        course_code: extract_str(xml_hm, "courseCode"),
        course_semester_hours: extract_i32(xml_hm, "courseSemesterHours"),
        group_id: extract_str(xml_hm, "groupID"),
        xgroup: extract_str(xml_hm, "group"),
        status_id: extract_str(xml_hm, "statusID").unwrap(),
        status: extract_str(xml_hm, "status").unwrap(),
        comment: extract_str(xml_hm, "comment").unwrap_or_default(),
        last_scrape: Utc::now().naive_utc(),
    }
}

fn extract_i32(hm: &HashMap<String, String>, key: &str) -> Option<i32> {
    let str = extract_str(hm, key)?;
    str.parse::<i32>().ok()
}

fn extract_dt(hm: &HashMap<String, String>, key: &str) -> Option<NaiveDateTime> {
    let str = extract_str(hm, key).unwrap();
    NaiveDateTime::parse_from_str(&str, "%Y%m%dT%H%M%S").ok()
}

fn extract_str(hm: &HashMap<String, String>, key: &str) -> Option<String> {
    hm.get(key).map(|s| s.trim().to_string())
}

pub struct XMLEvents {
    events: Vec<XMLEvent>,
}

const NS: &str = "http://rdm.campusonline.at/";

const CALENDAR_BASE_URL: &str =
    "https://campus.tum.de/tumonlinej/ws/webservice_v1.0/rdm/room/schedule/xml";

impl XMLEvents {
    pub(crate) fn len(&self) -> usize {
        self.events.len()
    }
    pub(crate) async fn store_in_db(self, conn: &PgPool) {
        for event in self.events {
            if let Err(e) = sqlx::query!(r#"
                INSERT INTO calendar(key, dtstart, dtend, dtstamp, event_id, event_title, single_event_id, single_event_type_id, single_event_type_name, event_type_id, event_type_name, course_type_name, course_type, course_code, course_semester_hours, group_id, xgroup, status_id, status, comment, last_scrape)
                VALUES               ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)"#,
                    event.key, event.dtstart, event.dtend, event.dtstamp, event.event_id, event.event_title, event.single_event_id, event.single_event_type_id, event.single_event_type_name, event.event_type_id, event.event_type_name, event.course_type_name, event.course_type, event.course_code, event.course_semester_hours, event.group_id, event.xgroup, event.status_id, event.status, event.comment, event.last_scrape)
                    .execute(conn).await {
                error!("Error inserting into database: {e:?}");
            }
        }
    }
    fn new(requested_room: &Room, body: &str) -> Option<Self> {
        let root = match body.parse::<Element>() {
            Ok(root) => root,
            Err(e) => {
                error!("Error parsing body to xml: {e:?} body={body:?}");
                return None;
            }
        };
        let mut events: Vec<XMLEvent> = Vec::new();
        let res = root.get_child("resource", NS).unwrap();
        let desc = res.get_child("description", NS).unwrap();
        let rg = desc.get_child("resourceGroup", NS).unwrap();
        let xml_super_events = rg.get_child("description", NS).unwrap();
        let xml_events = xml_super_events
            .children()
            .filter_map(|e| e.get_child("description", NS));

        for e in xml_events {
            let hm = construct_hm(e);

            let valid_status = match hm.get("status") {
                Some(s) => match s.as_str() {
                    "fix" | "geplant" => true,
                    "verschoben" | "gelöscht" | "abgesagt" | "abgelehnt" => false,
                    _ => {
                        error!("unknown status: {s:?}");
                        false
                    }
                },
                _ => false,
            };
            if valid_status {
                events.push(xml_event_from_hm(requested_room.sap_id.clone(), &hm));
            }
        }
        Some(XMLEvents { events })
    }
    pub(crate) async fn request(task: ScrapeRoomTask) -> Result<Self, Strategy> {
        // The token being embedded here is not an issue, since the token has only access to
        // the data this API is providing anyway...
        // If people want to disrupt this API, they can just do it by abusing this TUMonline-endpoint.
        // We (and TUMonline) monitor for this and will switch to a backup token not in this API
        // We do not want to repeat the DOS-attack that happened to TUMonline in December of 2022.
        let token = std::env::var("TUMONLINE_TOKEN")
            .unwrap_or_else(|_| "yeIKcuCGSzUCosnPZcKXkGeyUYGTQqUw".to_string());

        //get the xml file from TUMonline
        //why this API uses the tumonline_room_id and not the tumonline_calendar_id like the URLs is unclear
        //TUMonline apparently thinks this is sane
        let url = format!(
            "{CALENDAR_BASE_URL}?roomID={room_id}&timeMode=absolute&fromDate={from}&untilDate={to}&token={token}&buildingCode=",
            room_id=task.room.tumonline_room_id,
            from=task.from.format("%Y%m%d"),
            to=task.to.format("%Y%m%d")
        );
        debug!("url: {url}");
        for retry_cnt in 1..=5 {
            let body = request_body(url.to_string()).await;
            // randomized to avoid thundering herd phenomenon
            let mut rng = rand::thread_rng();
            // Retry 1: 400..800ms
            // Retry 5: 6.4s..12.8s
            let backoff_ms = rng.gen_range(2_u64.pow(retry_cnt)..2_u64.pow(retry_cnt + 1)) * 200;
            let backoff_duration = Duration::from_millis(backoff_ms);
            match body {
                RequestStatus::Success(body) => {
                    return XMLEvents::new(&task.room, &body).ok_or(Strategy::NoRetry);
                }
                // This consistently means, that there is no data for this room
                RequestStatus::NotFound => return Err(Strategy::NoRetry),
                // TUMonline sometimes returns an empty body due to one day being invalid
                //  => Retry smaller will get the other entries..
                RequestStatus::OneDayFaulty => return Err(Strategy::RetrySmaller),
                RequestStatus::Timeout | RequestStatus::Error => {
                    warn!("Retry {retry_cnt}/5, retrying in {backoff_duration:?} url={url}");
                }
            };
            sleep(backoff_duration).await;
        }
        // the entry may just be too large, as can be seen that we are getting enough Timeouts/Errors
        // => retrying smaller may be able to get around the errors/timeouts
        Err(Strategy::RetrySmaller)
    }
}

pub enum Strategy {
    NoRetry,
    RetrySmaller,
}