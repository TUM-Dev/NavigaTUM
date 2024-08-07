use crate::limited::vec::LimitedVec;
use polars::prelude::*;
use std::io::Write;
use tempfile::tempfile;

#[derive(Debug)]
pub(super) struct Alias {
    alias: String,
    key: String,    // the key is the id of the entry
    r#type: String, // what we display in the url
    visible_id: String,
}

impl Alias {
    async fn store(
        self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO aliases (alias, key, type, visible_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (alias,key) DO UPDATE SET
             key = EXCLUDED.key,
             type = EXCLUDED.type,
             visible_id = EXCLUDED.visible_id"#,
            self.alias,
            self.key,
            self.r#type,
            self.visible_id,
        )
        .execute(&mut **tx)
        .await
    }
}
#[tracing::instrument]
pub async fn download_updates() -> anyhow::Result<LimitedVec<Alias>> {
    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = reqwest::get(format!("{cdn_url}/api_data.parquet"))
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let mut aliase = Vec::<Alias>::new();
    let mut file = tempfile()?;
    file.write_all(&body)?;
    let df = ParquetReader::new(&mut file)
        .with_columns(Some(vec![
            "id".to_string(),
            "type".to_string(),
            "visible_id".to_string(),
            "aliases".to_string(),
        ]))
        .finish()
        .unwrap();
    let id_col = df.column("id")?.str()?;
    let type_col = df.column("type")?.str()?;
    let visible_id_col = df.column("visible_id")?.str()?;
    for index in 0..id_col.len() {
        let id = id_col.get(index).unwrap();
        let r#type = type_col.get(index).unwrap();
        let visible_id = visible_id_col.get(index);
        let visible_id = match visible_id {
            Some(v) => v.to_string(),
            None => id.to_string(),
        };
        aliase.push(Alias {
            alias: id.to_string(),
            key: id.to_string(),
            r#type: r#type.to_string(),
            visible_id: visible_id.clone(),
        });
        aliase.push(Alias {
            alias: visible_id.clone(),
            key: id.to_string(),
            r#type: r#type.to_string(),
            visible_id: visible_id.clone(),
        });
    }

    let df_expanded = df.explode(["aliases"])?;
    let mask = df_expanded.column("aliases")?.is_not_null();
    let df_expanded = df_expanded.filter(&mask)?;
    let id_col = df_expanded.column("id")?.str()?;
    let type_col = df_expanded.column("type")?.str()?;
    let visible_id_col = df_expanded.column("visible_id")?.str()?;
    let aliases_col = df_expanded.column("aliases")?.str()?;
    for index in 0..id_col.len() {
        let alias = aliases_col.get(index).unwrap();
        let id = id_col.get(index).unwrap();
        let r#type = type_col.get(index).unwrap();
        let visible_id = visible_id_col.get(index);
        let visible_id = match visible_id {
            Some(v) => v.to_string(),
            None => id.to_string(),
        };
        aliase.push(Alias {
            alias: alias.to_string(),
            key: id.to_string(),
            r#type: r#type.to_string(),
            visible_id,
        });
    }
    Ok(LimitedVec(aliase))
}
#[tracing::instrument(skip(tx))]
pub async fn load_all_to_db(
    aliases: LimitedVec<Alias>,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> anyhow::Result<()> {
    for task in aliases {
        task.store(tx).await?;
    }
    Ok(())
}
