mod alias;
mod data;

use sqlx::{Executor, SqlitePool};

pub(crate) async fn setup_database(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::migrate!("./migrations").run(pool).await?;
    // this is to setup the database faster
    // we don't want to use an acid compliant database for this step ;)
    pool.execute("PRAGMA journal_mode = OFF;");
    pool.execute("PRAGMA synchronous = OFF;");

    // delete all old data
    sqlx::query!("DELETE FROM aliases").execute(pool).await?;
    sqlx::query!("DELETE FROM en").execute(pool).await?;
    sqlx::query!("DELETE FROM de").execute(pool).await?;

    data::load_all_to_db(pool).await?;
    alias::load_all_to_db(pool).await?;
    Ok(())
}
