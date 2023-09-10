mod alias;
mod data;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Executor;

pub(crate) async fn setup_database() -> Result<(), Box<dyn std::error::Error>> {
    let uri = std::env::var("DB_LOCATION").unwrap_or_else(|_| "main-api/api_data.db".to_string());
    let uri = format!("{uri}?mode=rwc");
    let pool = SqlitePoolOptions::new().connect(&uri).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    // this is to setup the database faster
    // we don't want to use an acid compliant database for this step ;)
    pool.execute("PRAGMA journal_mode = OFF;");
    pool.execute("PRAGMA synchronous = OFF;");

    // delete all old data
    sqlx::query!("DELETE FROM aliases").execute(&pool).await?;
    sqlx::query!("DELETE FROM en").execute(&pool).await?;
    sqlx::query!("DELETE FROM de").execute(&pool).await?;

    data::load_all_to_db(&pool).await?;
    alias::load_all_to_db(&pool).await?;
    Ok(())
}
