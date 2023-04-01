use crate::scrape_task::ScrapeTask;
use log::info;

mod models;
mod schema;
mod scrape_task;
mod utils;

fn get_time_window_to_be_scraped() -> chrono::Duration {
    let time_window_months =
        std::env::var("SCRAPED_TIME_WINDOW_MONTHS").expect("SCRAPED_TIME_WINDOW_MONTHS not set");
    let time_window_months = time_window_months
        .parse::<i64>()
        .expect("SCRAPED_TIME_WINDOW_MONTHS not a number");
    // 30 days/month is a simplification, but over-scraping by a few days probably does not matter
    chrono::Duration::days(time_window_months * 30)
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let time_window = get_time_window_to_be_scraped();
    info!("Scraping time window: {}", time_window);
    let scraper = ScrapeTask::new(time_window);
    scraper.scrape_to_db().await;
    scraper.delete_stale_results();
}
