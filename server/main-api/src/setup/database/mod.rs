use log::info;

mod alias;
mod data;

pub async fn setup(pool: &sqlx::PgPool) -> Result<(), crate::BoxedError> {
    info!("setting up the database");
    sqlx::migrate!("./migrations").run(pool).await?;
    info!("migrations complete");

    let mut tx = pool.begin().await?;
    load_data(&mut tx).await?;
    tx.commit().await?;
    Ok(())
}
async fn load_data(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), crate::BoxedError> {
    info!("deleting old data");
    sqlx::query!("DELETE FROM aliases").execute(&mut **tx).await?;
    sqlx::query!("DELETE FROM en").execute(&mut **tx).await?;
    sqlx::query!("DELETE FROM de").execute(&mut **tx).await?;

    info!("loading new data");
    data::load_all_to_db(tx).await?;
    info!("loading new aliases");
    alias::load_all_to_db(tx).await?;
    Ok(())
}
