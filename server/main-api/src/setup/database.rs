use log::info;
use serde::Deserialize;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Executor, SqlitePool};

#[derive(Debug, Clone, Deserialize, Hash, PartialEq, Eq)]
struct Alias {
    alias: String,
    key: String,    // the key is the id of the entry
    r#type: String, // what we display in the url
    visible_id: String,
}

impl Alias {
    async fn load_all_to_db(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
        let inserted_aliases = 0;

        info!("Extracted {inserted_aliases} aliases");
        sqlx::query!(
            r#"INSERT OR REPLACE INTO aliases (alias, key, type, visible_id)
            VALUES (?, ?, ?, ?)"#,
            "abc",
            "dsa",
            "das",
            "visible_id"
        )
        .fetch_all(pool)
        .await?;

        Ok(())
    }
}

const DATABASE_URL: &str = "main-api/api_data.db?mode=rwc";
pub(crate) async fn setup_database() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePoolOptions::new().connect(DATABASE_URL).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    // this is to setup the database faster
    // we don't want to use an acid compliant database for this step ;)
    pool.execute("PRAGMA journal_mode = OFF;");
    pool.execute("PRAGMA synchronous = OFF;");

    // delete all onld data
    sqlx::query!(
        r#"DELETE FROM aliases;
    DELETE FROM de;
    DELETE FROM en;
    "#
    )
    .execute(&pool)
    .await?;

    Alias::load_all_to_db(&pool).await?;
    Ok(())
}
