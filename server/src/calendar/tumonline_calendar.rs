use crate::calendar::continous_scraping::ScrapeRoomTask;
use crate::{schema, utils};
use awc::error::{ConnectError, PayloadError, SendRequestError};
use awc::Client;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::Insertable;
use log::{debug, error, warn};
use minidom::Element;
use rand::Rng;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

enum RequestStatus {
    Success(String),
    Timeout,
    NotFound,
    TooLarge,
    OneDayFaulty,
    Error,
}
async fn request_body(client: &Client, url: String) -> RequestStatus {
    let req = client.get(&url).send().await;
    let body = match req {
        Ok(mut res) => match res.status().as_u16() {
            200 => res.body().limit(2_usize.pow(32)).await,
            404 => return RequestStatus::NotFound,
            _ => {
                error!("Error sending request (invalid status code): {:?}", res);
                return RequestStatus::Error;
            }
        },
        Err(e) => {
            return match e {
                // Timeouts are retried after a backoff
                SendRequestError::Timeout => RequestStatus::Timeout,
                SendRequestError::Connect(ConnectError::Timeout) => RequestStatus::Timeout,
                SendRequestError::H2(e) => {
                    if e.is_go_away() {
                        // for some pieces of work TUMonline somehow initally sends us this, but if we are persistent, it works...WTF?
                        return RequestStatus::Timeout;
                    }
                    error!("Error sending request: {:?}", e);
                    RequestStatus::Error
                }
                _ => {
                    error!("Error sending request: {:?}", e);
                    RequestStatus::Error
                }
            };
        }
    };
    let res_string = match body {
        Ok(body) => String::from_utf8(body.to_vec()),
        Err(PayloadError::Overflow) => {
            error!("RequestStatus::TooLarge => split and retry");
            return RequestStatus::TooLarge;
        }
        Err(e) => {
            error!("Error getting body: {:?}", e);
            return RequestStatus::Error;
        }
    };
    match res_string {
        Ok(res_string) => match res_string.as_str() {
            "" => RequestStatus::OneDayFaulty,
            _ => RequestStatus::Success(res_string),
        },
        Err(e) => {
            error!("Error converting body to string: {:?}", e);
            RequestStatus::Error
        }
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

#[derive(Insertable, Queryable)]
#[diesel(table_name = schema::calendar_scrape)]
pub struct XMLEvent {
    pub key: String,
    pub dtstart: NaiveDateTime,
    pub dtend: NaiveDateTime,
    pub dtstamp: NaiveDateTime,
    pub event_id: i32,
    pub event_title: String,
    pub single_event_id: i32,
    pub single_event_type_id: String,
    pub single_event_type_name: String,
    pub event_type_id: String,
    pub event_type_name: Option<String>,
    pub course_type_name: Option<String>,
    pub course_type: Option<String>,
    pub course_code: Option<String>,
    pub course_semester_hours: Option<i32>,
    pub group_id: Option<String>,
    pub xgroup: Option<String>,
    pub status_id: String,
    pub status: String,
    pub comment: String,
}

impl XMLEvent {
    fn from_hm(key: String, hm: HashMap<String, String>) -> XMLEvent {
        let other_keys = hm
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
            error!("found additional key(s) in hashmap: {:?}", other_keys);
        }
        XMLEvent {
            key,
            dtstart: extract_dt(&hm, "dtstart").unwrap(),
            dtend: extract_dt(&hm, "dtend").unwrap(),
            dtstamp: extract_dt(&hm, "dtstamp").unwrap(),
            event_id: extract_i32(&hm, "eventID").unwrap(),
            event_title: extract_str(&hm, "eventTitle").unwrap(),
            single_event_id: extract_i32(&hm, "singleEventID").unwrap(),
            single_event_type_id: extract_str(&hm, "singleEventTypeID").unwrap(),
            single_event_type_name: extract_str(&hm, "singleEventTypeName").unwrap(),
            event_type_id: extract_str(&hm, "eventTypeID").unwrap(),
            event_type_name: extract_str(&hm, "eventTypeName"),
            course_type_name: extract_str(&hm, "courseTypeName"),
            course_type: extract_str(&hm, "courseType"),
            course_code: extract_str(&hm, "courseCode"),
            course_semester_hours: extract_i32(&hm, "courseSemesterHours"),
            group_id: extract_str(&hm, "groupID"),
            xgroup: extract_str(&hm, "group"),
            status_id: extract_str(&hm, "statusID").unwrap(),
            status: extract_str(&hm, "status").unwrap(),
            comment: extract_str(&hm, "comment").unwrap_or_default(),
        }
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
pub(crate) struct XMLEvents {
    events: Vec<XMLEvent>,
}

const NS: &str = "http://rdm.campusonline.at/";

const CALENDAR_BASE_URL: &str =
    "https://campus.tum.de/tumonlinej/ws/webservice_v1.0/rdm/room/schedule/xml";

impl XMLEvents {
    pub(crate) fn len(&self) -> usize {
        self.events.len()
    }
    pub(crate) fn store_in_db(self) -> bool {
        let conn = &mut utils::establish_connection();
        use schema::calendar_scrape::dsl::*;
        let res = diesel::insert_into(calendar_scrape)
            .values(&self.events)
            .execute(conn);
        match res {
            Ok(_) => true,
            Err(e) => {
                error!("Error inserting into database: {:?}", e);
                false
            }
        }
    }
    fn new(key: String, body: String) -> Option<Self> {
        let root = body.parse::<Element>();
        let root = match root {
            Ok(root) => root,
            Err(e) => {
                error!("Error parsing body to xml: {:?} body={:?}", e, body);
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
            events.push(XMLEvent::from_hm(key.clone(), hm));
        }
        Some(XMLEvents { events })
    }
    pub(crate) async fn request(client: &Client, task: ScrapeRoomTask) -> Result<Self, Strategy> {
        // The token being embedded here is not an issue, since the token has only access to
        // the data this API is providing anyway...
        // If people want to disrupt this API, they can just do it by abusing this TUMonline-endpoint.
        // We (and TUMonline) monitor for this and will switch to a backup token not in this API
        // We do not want to repeat the DOS-attack that happened to TUMonline in December of 2022.
        let token = std::env::var("TUMONLINE_TOKEN")
            .unwrap_or_else(|_| "yeIKcuCGSzUCosnPZcKXkGeyUYGTQqUw".to_string());

        //get the xml file from TUMonline
        let url = format!(
            "{}?roomID={}&timeMode=absolute&fromDate={}&untilDate={}&token={}&buildingCode=",
            CALENDAR_BASE_URL,
            task.room_id,
            task.from.format("%Y%m%d"),
            task.to.format("%Y%m%d"),
            token
        );
        debug!("url: {}", url);
        for retry_cnt in 1..=5 {
            let body = request_body(client, url.to_string()).await;
            // randomized to avoid thundering herd phenomenon
            let mut rng = rand::thread_rng();
            // Retry 1: 400..800ms
            // Retry 5: 6.4s..12.8s
            let backoff_ms = rng.gen_range(2_u64.pow(retry_cnt)..2_u64.pow(retry_cnt + 1)) * 200;
            let backoff_duration = Duration::from_millis(backoff_ms);
            match body {
                RequestStatus::Success(body) => {
                    return XMLEvents::new(task.key, body).ok_or(Strategy::NoRetry);
                }
                // This consistently means, that there is no data for this room
                RequestStatus::NotFound => return Err(Strategy::NoRetry),
                // TUMonline sometimes returns an empty body due to one day being invalid
                //  => Retry smaller will get the other entries..
                RequestStatus::OneDayFaulty => return Err(Strategy::RetrySmaller),
                // We are requesting a lot of data. Sometimes too much => Retry smaller
                RequestStatus::TooLarge => return Err(Strategy::RetrySmaller),
                RequestStatus::Timeout | RequestStatus::Error => {
                    warn!(
                        "Retry {}/5, retrying in {:?} url={}",
                        retry_cnt, backoff_duration, url
                    );
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
