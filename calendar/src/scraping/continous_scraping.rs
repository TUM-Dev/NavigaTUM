use crate::scraping::tumonline_calendar::{Strategy, XMLEvents};
use crate::utils;
use crate::utils::statistics::Statistic;
use awc::{Client, Connector};
use chrono::{Datelike, NaiveDate};
use diesel::prelude::*;
use futures::future::join_all;
use log::{error, info, warn};
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub async fn start_scraping() {
    let mut interval = actix_rt::time::interval(Duration::from_secs(60 * 60 * 24)); //24h
    loop {
        interval.tick().await;
        delete_scraped_results();
        scrape_to_db(4).await;
        promote_scraped_results_to_prod();
    }
}

pub async fn scrape_to_db(year_duration: i32) {
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
    for round in all_room_ids.chunks(3) {
        i += 3;
        let round_start_time = Instant::now();
        let current_year = chrono::Utc::now().year();
        let mut futures = vec![];
        for (key, room_id) in round {
            futures.push(scrape(
                &client,
                (key.clone(), *room_id),
                current_year - year_duration / 2,
                year_duration,
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

#[derive(Clone, Debug)]
pub(crate) struct ScrapeRoomTask {
    pub(crate) key: String,
    pub(crate) room_id: i32,
    pub(crate) from: NaiveDate,
    pub(crate) to: NaiveDate,
}

impl ScrapeRoomTask {
    fn new((key, room_id): (String, i32), from_year: i32, year_duration: i32) -> Self {
        let from = NaiveDate::from_ymd_opt(from_year, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(from_year + year_duration, 1, 1).unwrap()
            - chrono::Days::new(1);
        Self {
            key,
            room_id,
            from,
            to,
        }
    }
    pub fn num_days(&self) -> u64 {
        // we want to count from the morning of "from" to the evening of "to" => +1
        (self.to + chrono::Days::new(1))
            .signed_duration_since(self.from)
            .num_days() as u64
    }
    fn split(&self) -> (Self, Self) {
        let mid_offset = self.num_days() / 2 - 1;
        let lower_middle = self.from + chrono::Days::new(mid_offset);
        (
            Self {
                key: self.key.clone(),
                room_id: self.room_id,
                from: self.from,
                to: lower_middle,
            },
            Self {
                key: self.key.clone(),
                room_id: self.room_id,
                from: lower_middle + chrono::Days::new(1),
                to: self.to,
            },
        )
    }
}

#[cfg(test)]
mod test_scrape_task {
    use super::ScrapeRoomTask;
    use chrono::NaiveDate;
    #[test]
    fn test_split() {
        let task = ScrapeRoomTask::new(("".to_string(), 0), 2020, 1);
        let (o1, o2) = task.split();
        assert_eq!(task.from, NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        assert_eq!(task.to, NaiveDate::from_ymd_opt(2020, 12, 31).unwrap());
        assert_eq!(o1.from, task.from);
        assert_eq!(o2.to, task.to);
        assert_eq!(o1.to + chrono::Duration::days(1), o2.from);
    }
    #[test]
    fn test_split_small() {
        let task = ScrapeRoomTask {
            key: "".to_string(),
            room_id: 0,
            from: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            to: NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
        };
        let (t1, t2) = task.split();
        assert_eq!(t1.to + chrono::Duration::days(1), t2.from);
        assert_eq!(task.num_days(), 2);
        assert_eq!(task.from, t1.from);
        assert_eq!(task.to, t2.to);
        assert_eq!(t1.num_days(), 1);
        assert_eq!(t2.num_days(), 1);
        assert_eq!(task.from, t1.to);
        assert_eq!(task.to, t2.from);
    }
    #[test]
    fn test_num_days() {
        let mut task = ScrapeRoomTask {
            key: "".to_string(),
            room_id: 0,
            from: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            to: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        };
        assert_eq!(task.num_days(), 1);
        task.to = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
        assert_eq!(task.num_days(), 2);
        task.to = NaiveDate::from_ymd_opt(2020, 12, 31).unwrap();
        assert_eq!(task.num_days(), 366);
    }
    #[test]
    fn test_same_day() {
        let task = ScrapeRoomTask::new(("".to_string(), 0), 2020, 0);
        assert_eq!(task.from, NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        assert_eq!(task.to, NaiveDate::from_ymd_opt(2019, 12, 31).unwrap());
        assert_eq!(task.num_days(), 0);
    }
}

struct ScrapeResult {
    retry_smaller_happened: bool,
    success_cnt: usize,
}

async fn scrape(
    client: &Client,
    id: (String, i32),
    from_year: i32,
    year_duration: i32,
) -> ScrapeResult {
    // request and parse the xml file
    let mut request_queue = vec![ScrapeRoomTask::new(id, from_year, year_duration)];
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
