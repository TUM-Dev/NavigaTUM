mod data;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Executor;

const DATABASE_URL: &str = "main-api/api_data.db?mode=rwc";
pub(crate) async fn setup_database() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePoolOptions::new().connect(DATABASE_URL).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    // this is to setup the database faster
    // we don't want to use an acid compliant database for this step ;)
    pool.execute("PRAGMA journal_mode = OFF;");
    pool.execute("PRAGMA synchronous = OFF;");

    // delete all old data
    sqlx::query!("DELETE FROM aliases").execute(&pool).await?;
    sqlx::query!("DELETE FROM de").execute(&pool).await?;
    sqlx::query!("DELETE FROM en").execute(&pool).await?;

    data::load_all_to_db(&pool).await?;
    Ok(())
}
