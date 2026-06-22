use tracing::{Instrument as _, debug, info, info_span};

use crate::limited::vec::LimitedVec;

mod alias;
mod data;
mod osm;

#[tracing::instrument(skip(pool))]
pub async fn setup(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    info!("setting up the database");
    sqlx::migrate!("./migrations").run(pool).await?;
    info!("migrations complete");
    Ok(())
}
#[tracing::instrument(skip(pool))]
pub async fn load_data(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    debug!("starting to download the status");
    let (new_keys, new_hashes) = data::download_status().await?;
    debug!("loaded new keys/hashes successfully");
    async {
        let mut tx = pool.begin().await?;
        cleanup_deleted(&new_keys, &mut tx).await?;
        tx.commit().await?;
        anyhow::Ok(())
    }
    .instrument(info_span!("deleting old data"))
    .await?;
    let keys_which_need_updating =
        find_keys_which_need_updating(pool, &new_keys, &new_hashes).await?;
    if !keys_which_need_updating.is_empty() {
        async {
            let data = data::download_updates(&keys_which_need_updating).await?;
            let mut tx = pool.begin().await?;
            data::load_all_to_db(data, &mut tx).await?;
            tx.commit().await?;
            anyhow::Ok(())
        }
        .instrument(info_span!("loading changed data"))
        .await?;
    }
    osm::override_room_coords(pool)
        .instrument(info_span!("overriding coordinates from OpenStreetMap"))
        .await?;
    {
        let aliases = alias::download_updates().await?;
        let mut tx = pool.begin().await?;
        alias::load_all_to_db(aliases, &mut tx).await?;
        tx.commit().await?;
    }
    Ok(())
}

#[tracing::instrument(skip(pool))]
pub(crate) async fn find_keys_which_need_updating(
    pool: &sqlx::PgPool,
    keys: &LimitedVec<String>,
    hashes: &LimitedVec<i64>,
) -> anyhow::Result<LimitedVec<String>> {
    let keys_which_need_updating = sqlx::query_scalar!(
        r#"
SELECT expected.key AS "key!"
FROM (SELECT * FROM UNNEST($1::text[], $2::int8[])) as expected(key,hash)
LEFT JOIN de ON de.key = expected.key
WHERE de.key IS NULL OR de.hash != expected.hash
"#,
        keys.as_ref(),
        hashes.as_ref(),
    )
    .fetch_all(pool)
    .await?;
    debug!(cnt = keys_which_need_updating.len(), "keys to (re)load");
    Ok(LimitedVec(keys_which_need_updating))
}

#[tracing::instrument(skip(tx))]
async fn cleanup_deleted(
    keys: &LimitedVec<String>,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> anyhow::Result<()> {
    let keys = &keys.0;
    sqlx::query!(
        "DELETE FROM aliases WHERE NOT EXISTS (SELECT * FROM UNNEST($1::text[]) AS expected(key) WHERE aliases.key = expected.key)",
        keys
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query!(
        "DELETE FROM en WHERE NOT EXISTS (SELECT * FROM UNNEST($1::text[]) AS expected(key) WHERE en.key = expected.key)",
        keys
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query!(
        "DELETE FROM calendar WHERE NOT EXISTS (SELECT * FROM UNNEST($1::text[]) AS expected(key) WHERE calendar.room_code = expected.key)",
        keys
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query!(
        "DELETE FROM de WHERE NOT EXISTS (SELECT * FROM UNNEST($1::text[]) AS expected(key) WHERE de.key = expected.key)",
        keys
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
