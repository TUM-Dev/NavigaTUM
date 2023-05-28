mod main_api_connector;
mod scrape_room_task;
pub mod tumonline_calendar_connector;

use crate::scrape_task::main_api_connector::{get_all_ids, Room};
use crate::scrape_task::scrape_room_task::ScrapeRoomTask;
use crate::scrape_task::tumonline_calendar_connector::{Strategy, XMLEvents};
use crate::utils;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use lazy_static::lazy_static;
use log::{info, warn};
use prometheus::{register_counter, register_histogram, Counter, Histogram};
use std::time::{Duration, Instant};
use tokio::time::sleep;
lazy_static! {
    static ref SCRAPED_CALENDAR_ENTRIES_COUNTER: Counter = register_counter!(
        "navigatum_calendarscraper_total_entries",
        "Total number of calendar entries scraped."
    )
    .unwrap();
    static ref REQ_SUCCESS_HISTOGRAM: Histogram = register_histogram!(
        "navigatum_calendarscraper_resulting_entries_buckets",
        "Amount of entries retrieved",
        prometheus::exponential_buckets(10.0, 2.0, 15).unwrap(),
    )
    .unwrap();
    static ref REQ_TIME_HISTOGRAM: Histogram = register_histogram!(
        "navigatum_calendarscraper_request_duration_ms_buckets",
        "The scrape request latencies in seconds",
        prometheus::linear_buckets(20.0, 20.0, 15).unwrap(),
    )
    .unwrap();
}

pub struct ScrapeTask {
    time_window: chrono::Duration,
    scraping_start: DateTime<Utc>,
}

const CONCURRENT_REQUESTS: usize = 2;
impl ScrapeTask {
    pub fn new(time_window: chrono::Duration) -> Self {
        Self {
            time_window,
            scraping_start: Utc::now(),
        }
    }

    pub async fn scrape_to_db(&self) {
        info!("Starting scraping calendar entries");
        let start_time = Instant::now();

        let mut all_room_ids = get_all_ids().await;
        let entry_cnt = all_room_ids.len();

        let mut work_queue = FuturesUnordered::new();
        let start = self.scraping_start - self.time_window / 2;
        while !all_room_ids.is_empty() {
            while work_queue.len() < CONCURRENT_REQUESTS {
                if let Some(room) = all_room_ids.pop() {
                    // sleep to not overload TUMonline.
                    // It is critical for successfully scraping that we are not blocked.
                    sleep(Duration::from_millis(50)).await;

                    work_queue.push(scrape(room, start.date_naive(), self.time_window));
                }
            }
            work_queue.next().await;

            let scraped_entries = entry_cnt - all_room_ids.len();
            if scraped_entries % 30 == 0 {
                let progress = scraped_entries as f32 / entry_cnt as f32 * 100.0;
                let elapsed = start_time.elapsed();
                let time_per_key = elapsed / scraped_entries as u32;
                info!("Scraped {progress:.2}% ({scraped_entries}/{entry_cnt}) in {elapsed:.1?} (avg {time_per_key:.1?}/key)");
            }
        }

        info!(
            "Finished scraping calendar entrys. ({entry_cnt} entries in {:?})",
            start_time.elapsed()
        );
    }

    pub fn delete_stale_results(&self) {
        use crate::schema::calendar::dsl::*;
        let start_time = Instant::now();
        let scrapeinterval = (
            self.scraping_start - self.time_window / 2,
            self.scraping_start + self.time_window / 2,
        );
        let conn = &mut utils::establish_connection();
        diesel::delete(calendar)
            .filter(dtstart.gt(scrapeinterval.0.naive_local()))
            .filter(dtend.le(scrapeinterval.1.naive_local()))
            .filter(last_scrape.le(self.scraping_start.naive_local()))
            .execute(conn)
            .expect("Failed to delete calendar");

        info!(
            "Finished deleting stale results ({time_window} in {passed:?})",
            time_window = self.time_window,
            passed = start_time.elapsed(),
        );
    }
}

async fn scrape(room: Room, from: NaiveDate, duration: chrono::Duration) {
    let _timer = REQ_TIME_HISTOGRAM.start_timer(); // drop as observe

    // request and parse the xml file
    let mut request_queue = vec![ScrapeRoomTask::new(room, from, duration)];
    let mut success_cnt = 0;
    while !request_queue.is_empty() {
        let mut new_request_queue = vec![];
        for task in request_queue {
            let events = XMLEvents::request(task.clone()).await;

            //store the events in the database if successful, otherwise retry
            match events {
                Ok(events) => {
                    success_cnt += events.len();
                    events.store_in_db();
                }
                Err(retry) => match retry {
                    Strategy::NoRetry => {}
                    Strategy::RetrySmaller => {
                        if task.num_days() > 1 {
                            let (t1, t2) = task.split();
                            new_request_queue.push(t1);
                            new_request_queue.push(t2);
                        } else {
                            warn!("The following ScrapeOrder cannot be fulfilled: {task:?}");
                        }
                    }
                },
            };

            // sleep to not overload TUMonline.
            // It is critical for successfully scraping that we are not blocked.
            sleep(Duration::from_millis(50)).await;
        }
        request_queue = new_request_queue;
    }

    REQ_SUCCESS_HISTOGRAM.observe(success_cnt as f64);
    SCRAPED_CALENDAR_ENTRIES_COUNTER.inc_by(success_cnt as f64);
}
