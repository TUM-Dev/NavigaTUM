use std::time::Instant;

use log::{debug, info};

mod alias;
mod data;

pub async fn setup(pool: &sqlx::PgPool) -> Result<(), crate::BoxedError> {
    info!("setting up the database");
    sqlx::migrate!("./migrations").run(pool).await?;
    info!("migrations complete");
    Ok(())
}
pub async fn load_data(pool: &sqlx::PgPool) -> Result<(), crate::BoxedError> {
    let status = data::download_status().await?;
    let new_keys = status
        .clone()
        .into_iter()
        .map(|(k, _)| k)
        .collect::<Vec<String>>();
    let new_hashes = status.into_iter().map(|(_, h)| h).collect::<Vec<i64>>();
    info!("deleting old data");
    {
        let start = Instant::now();
        let mut tx = pool.begin().await?;
        cleanup_deleted(&new_keys, &mut tx).await?;
        tx.commit().await?;
        debug!("deleted old data in {elapsed:?}", elapsed = start.elapsed());
    }

    debug!("finding changed data");
    let keys_which_need_updating =
        find_keys_which_need_updating(pool, &new_keys, &new_hashes).await?;

    if !keys_which_need_updating.is_empty() {
        info!("loading changed {} data", keys_which_need_updating.len());
        let data = data::download_updates(&keys_which_need_updating).await?;
        let mut tx = pool.begin().await?;
        data::load_all_to_db(data, &mut tx).await?;
        tx.commit().await?;
    }

    if !keys_which_need_updating.is_empty() {
        info!("loading new aliases");
        let aliases = alias::download_updates(&keys_which_need_updating).await?;
        let mut tx = pool.begin().await?;
        alias::load_all_to_db(aliases, &mut tx).await?;
        tx.commit().await?;
    }
    Ok(())
}

async fn find_keys_which_need_updating(
    pool: &sqlx::PgPool,
    keys: &[String],
    hashes: &[i64],
) -> Result<Vec<String>, crate::BoxedError> {
    let start = Instant::now();
    let mut keys_which_need_updating = sqlx::query_scalar!(
        r#"
SELECT de.key
FROM de, (SELECT * FROM UNNEST($1::text[], $2::int8[])) as expected(key,hash)
WHERE de.key = expected.key and de.hash != expected.hash
"#,
        keys,
        hashes
    )
    .fetch_all(pool)
    .await?;
    debug!("find_keys_which_need_updating (update) took {elapsed:?} and yielded {updated_cnt} updated items", elapsed = start.elapsed(), updated_cnt=keys_which_need_updating.len());

    let mut keys_which_need_removing = sqlx::query_scalar!(
        r#"
SELECT de.key
FROM de
WHERE NOT EXISTS (SELECT * FROM UNNEST($1::text[]) as expected2(key) where de.key=expected2.key)
"#,
        keys
    )
    .fetch_all(pool)
    .await?;
    debug!("find_keys_which_need_updating (update+delete) took {elapsed:?} and yielded {deleted_cnt} deleted items", elapsed = start.elapsed(), deleted_cnt=keys_which_need_removing.len());
    keys_which_need_updating.append(&mut keys_which_need_removing);
    Ok(keys_which_need_updating)
}

async fn cleanup_deleted(
    keys: &[String],
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), crate::BoxedError> {
    sqlx::query!("DELETE FROM aliases WHERE NOT EXISTS (SELECT * FROM UNNEST($1::text[]) AS expected(key) WHERE aliases.key = expected.key)", keys)
        .execute(&mut **tx)
        .await?;
    sqlx::query!("DELETE FROM en WHERE NOT EXISTS (SELECT * FROM UNNEST($1::text[]) AS expected(key) WHERE en.key = expected.key)", keys)
        .execute(&mut **tx)
        .await?;
    sqlx::query!("DELETE FROM de WHERE NOT EXISTS (SELECT * FROM UNNEST($1::text[]) AS expected(key) WHERE de.key = expected.key)", keys)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
