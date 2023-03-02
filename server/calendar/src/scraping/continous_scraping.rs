use crate::scraping::main_api::get_all_ids;
use crate::scraping::task::ScrapeRoomTask;
use crate::scraping::tumonline_calendar::{Strategy, XMLEvents};
use crate::utils;
use crate::utils::statistics::Statistic;
use awc::{Client, Connector};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use futures::future::join_all;
use log::{info, warn};
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub async fn start_scraping(scrape_every: Duration, duration: chrono::Duration) {
    let mut interval = actix_rt::time::interval(scrape_every);
    loop {
        interval.tick().await;
        let scraping_start = Utc::now();
        scrape_to_db(&scraping_start, duration).await;
        delete_stale_results(scraping_start, duration);
    }
}

pub async fn scrape_to_db(scraping_start: &DateTime<Utc>, duration: chrono::Duration) {
    info!("Starting scraping calendar entries");
    let start_time = Instant::now();

    // timeout is possibly excessive, this is tbd
    // Reasoning is, that a large timeout does not hinder us that much, as we retry
    let connector = Connector::new().timeout(Duration::from_secs(20));
    let client = Client::builder()
        .connector(connector)
        .timeout(Duration::from_secs(20))
        .finish();
    let all_room_ids = get_all_ids().await;
    let entry_cnt = all_room_ids.len();
    let mut time_stats = Statistic::new();
    let mut entry_stats = Statistic::new();

    let mut i = 0;
    for round in all_room_ids.chunks(2) {
        i += round.len();
        let round_start_time = Instant::now();
        let mut futures = vec![];
        for room in round {
            let start = *scraping_start - duration / 2;
            futures.push(scrape(
                &client,
                (room.key.clone(), room.tumonline_room_nr),
                start.date_naive(),
                duration,
            ));
        }
        let results: Vec<ScrapeResult> = join_all(futures).await;
        results
            .iter()
            .for_each(|e| entry_stats.push(e.success_cnt as u32));
        // if one of the futures needed to be retried smaller, this would skew the stats a lot
        if results.iter().all(|e| !e.retry_smaller_happened) {
            time_stats.push(round_start_time.elapsed());
        }
        if i % 30 == 0 {
            let progress = i as f32 / entry_cnt as f32 * 100.0;
            let elapsed = start_time.elapsed();
            let time_per_key = elapsed / i as u32;
            info!("Scraped {progress:.2}% (avg {time_per_key:.1?}/key, total {elapsed:.1?}) result-{entry_stats:?} in time-{time_stats:.1?}");
        }
        // sleep to not overload TUMonline.
        // It is critical for successfully scraping that we are not blocked.
        sleep(Duration::from_millis(100)).await;
    }

    info!(
        "Finished scraping calendar entrys. ({entry_cnt} entries in {:?})",
        start_time.elapsed()
    );
}

fn delete_stale_results(scraping_start: DateTime<Utc>, duration: chrono::Duration) {
    use crate::schema::calendar::dsl::*;
    let start_time = Instant::now();
    let scrapeinterval = (scraping_start - duration / 2, scraping_start + duration / 2);
    let conn = &mut utils::establish_connection();
    diesel::delete(calendar)
        .filter(dtstart.gt(scrapeinterval.0.naive_local()))
        .filter(dtend.le(scrapeinterval.1.naive_local()))
        .filter(last_scrape.le(scraping_start.naive_local()))
        .execute(conn)
        .expect("Failed to delete calendar");

    let passed = start_time.elapsed();
    info!("Finished deleting stale results ({duration} in {passed:?})");
}

struct ScrapeResult {
    retry_smaller_happened: bool,
    success_cnt: usize,
}

async fn scrape(
    client: &Client,
    id: (String, i32),
    from: NaiveDate,
    duration: chrono::Duration,
) -> ScrapeResult {
    // request and parse the xml file
    let mut request_queue = vec![ScrapeRoomTask::new(id, from, duration)];
    let mut success_cnt = 0;
    let mut retry_smaller_happened = false;
    while !request_queue.is_empty() {
        let mut new_request_queue = vec![];
        for task in request_queue {
            let events = XMLEvents::request(client, task.clone()).await;

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
                        retry_smaller_happened = true;
                    }
                },
            };

            // sleep to not overload TUMonline.
            // It is critical for successfully scraping that we are not blocked.
            sleep(Duration::from_millis(50)).await;
        }
        request_queue = new_request_queue;
    }
    ScrapeResult {
        retry_smaller_happened,
        success_cnt,
    }
}
