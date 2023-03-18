use crate::scrape_task::ScrapeTask;

mod models;
mod schema;
mod scrape_task;
mod utils;

#[tokio::main]
async fn main() {
    let time_window_months =
        std::env::var("SCRAPED_TIME_WINDOW_MONTHS").expect("SCRAPED_TIME_WINDOW_MONTHS not set");
    let time_window_months = time_window_months
        .parse::<i64>()
        .expect("SCRAPED_TIME_WINDOW_MONTHS not a number");
    // 30 days/month is a simplification, but over-scraping by a few days probably does not matter
    let time_window = chrono::Duration::days(time_window_months * 30);
    let scraper = ScrapeTask::new(time_window);
    scraper.scrape_to_db().await;
    scraper.delete_stale_results();
}
