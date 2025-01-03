use tracing::{debug, debug_span, info, info_span};

use crate::limited::vec::LimitedVec;

mod alias;
mod data;

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
    {
        let _ = info_span!("deleting old data").enter();
        let mut tx = pool.begin().await?;
        cleanup_deleted(&new_keys, &mut tx).await?;
        tx.commit().await?;
    }
    let keys_which_need_updating =
        find_keys_which_need_updating(pool, &new_keys, &new_hashes).await?;
    if !keys_which_need_updating.is_empty() {
        let _ = info_span!("loading changed data").enter();
        let data = data::download_updates(&keys_which_need_updating).await?;
        let mut tx = pool.begin().await?;
        data::load_all_to_db(data, &mut tx).await?;
        tx.commit().await?;
    }
    {
        let aliases = alias::download_updates().await?;
        let mut tx = pool.begin().await?;
        alias::load_all_to_db(aliases, &mut tx).await?;
        tx.commit().await?;
    }
    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn find_keys_which_need_updating(
    pool: &sqlx::PgPool,
    keys: &LimitedVec<String>,
    hashes: &LimitedVec<i64>,
) -> anyhow::Result<LimitedVec<String>> {
    let number_of_keys = sqlx::query_scalar!("SELECT COUNT(*) FROM de")
        .fetch_one(pool)
        .await?;
    if number_of_keys == Some(0) {
        debug!(cnt = keys.len(), "all keys need updating",);
        return Ok(keys.clone());
    }

    let mut keys_which_need_updating = {
        let _ = debug_span!("keys_which_need_updating").enter();
        let keys_which_need_updating = sqlx::query_scalar!(
            r#"
SELECT de.key
FROM de, (SELECT * FROM UNNEST($1::text[], $2::int8[])) as expected(key,hash)
WHERE de.key = expected.key and de.hash != expected.hash
"#,
            keys.as_ref(),
            hashes.as_ref(),
        )
        .fetch_all(pool)
        .await?;
        debug!(cnt = keys_which_need_updating.len(), "updated items",);
        keys_which_need_updating
    };

    let mut keys_which_need_removing = {
        let _ = debug_span!("keys_which_need_removing").enter();
        let keys_which_need_removing = sqlx::query_scalar!(
            r#"
SELECT de.key
FROM de
WHERE NOT EXISTS (SELECT * FROM UNNEST($1::text[]) as expected2(key) where de.key=expected2.key)
"#,
            keys.as_ref()
        )
        .fetch_all(pool)
        .await?;
        debug!(cnt = keys_which_need_removing.len(), "deleted items",);
        keys_which_need_removing
    };
    keys_which_need_updating.append(&mut keys_which_need_removing);
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
