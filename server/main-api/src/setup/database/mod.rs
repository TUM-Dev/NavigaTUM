use log::info;

mod alias;
mod data;

pub async fn setup(pool: &sqlx::PgPool) -> Result<(), crate::BoxedError> {
    info!("setting up the database");
    sqlx::migrate!("./migrations").run(pool).await?;
    info!("migrations complete");
    Ok(())
}
pub async fn load_data(pool: &sqlx::PgPool) -> Result<(), crate::BoxedError> {
    let mut tx = pool.begin().await?;

    info!("deleting old data");
    cleanup(&mut tx).await?;
    info!("loading new data");
    data::load_all_to_db(&mut tx).await?;
    info!("loading new aliases");
    alias::load_all_to_db(&mut tx).await?;
    tx.commit().await?;
    Ok(())
}

async fn cleanup(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<(), crate::BoxedError> {
    sqlx::query!("DELETE FROM aliases")
        .execute(&mut **tx)
        .await?;
    sqlx::query!("DELETE FROM en").execute(&mut **tx).await?;
    sqlx::query!("DELETE FROM de").execute(&mut **tx).await?;
    Ok(())
}
