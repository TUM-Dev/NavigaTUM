mod alias;
mod data;

use log::info;
use sqlx::PgPool;

pub(crate) async fn setup_database(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::migrate!("./migrations").run(pool).await?;

    info!("database setup complete, deleting old data");
    sqlx::query!("DELETE FROM aliases").execute(pool).await?;
    sqlx::query!("DELETE FROM en").execute(pool).await?;
    sqlx::query!("DELETE FROM de").execute(pool).await?;

    info!("loading new data");
    data::load_all_to_db(pool).await?;
    info!("loading new aliases");
    alias::load_all_to_db(pool).await?;
    Ok(())
}
