use std::ops::Sub;

use actix_web::HttpResponse;
use chrono::{DateTime, FixedOffset, Utc};
use log::error;
use sqlx::PgPool;

use connectum::APIRequestor;
use db::DbRequestor;

use crate::calendar::models::Event;

mod connectum;
mod db;

type CalendarEntries = (DateTime<Utc>, Vec<Event>);

trait CalendarEntryFetcher {
    fn new(pool: &PgPool, last_calendar_scrape_at: &Option<DateTime<Utc>>) -> Self;
    async fn fetch(
        &self,
        id: &str,
        start_after: &DateTime<Utc>,
        end_before: &DateTime<Utc>,
    ) -> Result<CalendarEntries, crate::BoxedError>;
}

pub struct StrategyExecutor {
    pool: PgPool,
    id: String,
    start_after: DateTime<Utc>,
    end_before: DateTime<Utc>,
}

impl StrategyExecutor {
    pub(super) fn new(
        pool: &PgPool,
        id: &str,
        start_after: &DateTime<Utc>,
        end_before: &DateTime<Utc>,
    ) -> Self {
        Self {
            pool: pool.clone(),
            id: id.into(),
            start_after: *start_after,
            end_before: *end_before,
        }
    }
    async fn exec<T: CalendarEntryFetcher>(
        &self,
        last_calendar_scrape_at: &Option<DateTime<Utc>>,
    ) -> Result<CalendarEntries, crate::BoxedError> {
        T::new(&self.pool, last_calendar_scrape_at)
            .fetch(&self.id, &self.start_after, &self.end_before)
            .await
    }

    pub(super) async fn exec_with_retrying(
        self,
        last_calendar_scrape_at: &Option<DateTime<Utc>>,
    ) -> Result<CalendarEntries, HttpResponse> {
        let intial = match last_calendar_scrape_at {
            Some(l) => {
                if Self::one_hour_ago() < *l {
                    self.exec::<APIRequestor>(last_calendar_scrape_at).await
                } else {
                    self.exec::<DbRequestor>(last_calendar_scrape_at).await
                }
            }
            None => self.exec::<APIRequestor>(last_calendar_scrape_at).await,
        };

        match intial {
            Ok(r) => Ok(r),
            Err(e) => {
                error!("could not fetch due to {e:?}");
                let last_scrape = last_calendar_scrape_at.unwrap_or_default();
                if Self::three_days_ago() < last_scrape {
                    match self.exec::<DbRequestor>(last_calendar_scrape_at).await {
                        Ok(res) => Ok(res),
                        Err(e) => {
                            error!("could not get substitute from db due to {e:?}");
                            Err(HttpResponse::InternalServerError()
                                .body("could not get calendar entrys, please try again later"))
                        }
                    }
                } else {
                    error!("cannot get substitute from db due to staleness");
                    Err(HttpResponse::InternalServerError()
                        .body("could not get calendar entrys, please try again later"))
                }
            }
        }
    }

    fn one_hour_ago() -> DateTime<Utc> {
        let one_hour = FixedOffset::east_opt(60 * 60)
            .expect("time travel is impossible and chronos is Y2K38-safe");
        Utc::now().sub(one_hour)
    }

    fn three_days_ago() -> DateTime<Utc> {
        let three_days = chrono::Days::new(3);
        Utc::now()
            .checked_sub_days(three_days)
            .expect("time travel is impossible and chronos is Y2K38-save")
    }
}
