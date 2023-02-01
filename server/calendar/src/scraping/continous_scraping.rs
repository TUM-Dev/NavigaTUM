use crate::scraping::task::ScrapeRoomTask;
use crate::scraping::tumonline_calendar::{Strategy, XMLEvents};
use crate::utils;
use crate::utils::statistics::Statistic;
use actix_web::web::Data;
use awc::{Client, Connector};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use diesel::prelude::*;
use futures::future::join_all;
use log::{error, info, warn};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

const SECONDS_PER_DAY: u64 = 60 * 60 * 24;
pub async fn start_scraping(last_sync: Data<Mutex<Option<NaiveDateTime>>>) {
    let mut interval = actix_rt::time::interval(Duration::from_secs(SECONDS_PER_DAY)); //24h
    loop {
        interval.tick().await;
        delete_scraped_results();
        scrape_to_db(chrono::Duration::days(30 * 4)).await;
        promote_scraped_results_to_prod();
        {
            let mut last_scrape = last_sync.lock().await;
            *last_scrape = Some(Utc::now().naive_utc());
        }
    }
}

pub async fn scrape_to_db(duration: chrono::Duration) {
    info!("Starting scraping calendar entrys");
    let start_time = Instant::now();

    // timeout is possibly excessive, this is tbd
    // Reasoning is, that a large timeout does not hinder us that much, as we retry
    let connector = Connector::new().timeout(Duration::from_secs(20));
    let client = Client::builder()
        .connector(connector)
        .timeout(Duration::from_secs(20))
        .finish();
    let all_room_ids: Vec<(String, i32)> = get_all_ids(); // Vec<(key,tumonline_id)>
    let entry_cnt = all_room_ids.len();
    let mut time_stats = Statistic::new();
    let mut entry_stats = Statistic::new();

    let mut i = 0;
    for round in all_room_ids.chunks(2) {
        i += 2;
        let round_start_time = Instant::now();
        let mut futures = vec![];
        for (key, room_id) in round {
            let start = Utc::now() - duration / 2;
            futures.push(scrape(
                &client,
                (key.clone(), *room_id),
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
            info!(
                "Scraped {:.2}% (avg {:.1?}/key, total {:.1?}) result-{:?} in time-{:.1?}",
                i as f32 / entry_cnt as f32 * 100.0,
                start_time.elapsed() / i,
                start_time.elapsed(),
                entry_stats,
                time_stats,
            );
        }
        // sleep to not overload TUMonline.
        // It is critical for successfully scraping that we are not blocked.
        sleep(Duration::from_millis(100)).await;
    }

    info!(
        "Finished scraping calendar entrys. ({} entries in {}s)",
        entry_cnt,
        start_time.elapsed().as_secs_f32()
    );
}

fn delete_scraped_results() {
    let conn = &mut utils::establish_connection();
    use crate::schema::calendar_scrape::dsl::calendar_scrape;
    diesel::delete(calendar_scrape)
        .execute(conn)
        .expect("Failed to delete calendar");
}

fn promote_scraped_results_to_prod() {
    let start_time = Instant::now();
    let conn = &mut utils::establish_connection();
    use crate::schema::calendar::dsl::calendar;
    use crate::schema::calendar_scrape::dsl::{calendar_scrape, status};
    diesel::delete(calendar)
        .execute(conn)
        .expect("Failed to delete calendar");
    diesel::insert_into(calendar)
        .values(
            calendar_scrape
                .filter(status.eq("fix"))
                .or_filter(status.eq("geplant")),
        )
        .execute(conn)
        .expect("Failed to insert newly scraped values into db");

    info!(
        "Finished switching scraping results - prod. ({}s)",
        start_time.elapsed().as_secs_f32()
    );
}

fn get_all_ids() -> Vec<(String, i32)> {
    let conn = &mut utils::establish_connection();

    use crate::schema::de::dsl::*;
    // order is just here, to make debugging more reproducible. Performance impact is engligable
    let res = de
        .select((key, tumonline_room_nr))
        .filter(tumonline_room_nr.is_not_null())
        .order_by((key, tumonline_room_nr))
        .load::<(String, Option<i32>)>(conn);
    match res {
        Ok(d) => d
            .iter()
            .map(|(k, t)| (k.clone(), t.unwrap()))
            .collect::<Vec<(String, i32)>>(),
        Err(e) => {
            error!("Error requesting all ids: {:?}", e);
            vec![]
        }
    }
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
                            warn!("The following ScrapeOrder cannot be fulfilled: {:?}", task);
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
