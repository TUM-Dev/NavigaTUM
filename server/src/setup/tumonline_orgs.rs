use crate::setup::file_loader;
use polars::prelude::*;
use std::io::Write;
use tempfile::tempfile;

struct DBOrg {
    org_id: i32,
    code: String,
    name_de: String,
    name_en: String,
    path_de: Option<String>,
    path_en: Option<String>,
}

impl DBOrg {
    async fn store(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO tumonline_orgs(org_id, code, name_de, name_en, path_de, path_en) \
             VALUES ($1, $2, $3, $4, $5, $6)",
            self.org_id,
            self.code,
            self.name_de,
            self.name_en,
            self.path_de,
            self.path_en,
        )
        .execute(&mut **tx)
        .await
    }
}

#[tracing::instrument(skip(pool))]
pub async fn setup(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("tumonline_orgs.parquet", &cdn_url).await?;

    let mut file = tempfile()?;
    file.write_all(&body)?;

    let df = ParquetReader::new(&mut file).finish()?;

    let org_id_col = df.column("org_id")?.i32()?;
    let code_col = df.column("code")?.str()?;
    let name_de_col = df.column("name_de")?.str()?;
    let name_en_col = df.column("name_en")?.str()?;
    let path_de_col = df.column("path_de")?.str()?;
    let path_en_col = df.column("path_en")?.str()?;

    let mut orgs = Vec::with_capacity(df.height());
    for i in 0..df.height() {
        let Some(org_id) = org_id_col.get(i) else {
            continue;
        };
        let Some(code) = code_col.get(i) else {
            continue;
        };
        let Some(name_en) = name_en_col.get(i) else {
            continue;
        };
        let name_de = name_de_col.get(i).unwrap_or(name_en);

        orgs.push(DBOrg {
            org_id,
            code: code.to_string(),
            name_de: name_de.to_string(),
            name_en: name_en.to_string(),
            path_de: path_de_col.get(i).map(str::to_string),
            path_en: path_en_col.get(i).map(str::to_string),
        });
    }

    // Truncate cascades to events because events.organising_org_id references tumonline_orgs(org_id).
    // Order matters: tumonline_orgs::setup must run before events::setup.
    let mut tx = pool.begin().await?;
    sqlx::query!("TRUNCATE TABLE tumonline_orgs RESTART IDENTITY CASCADE")
        .execute(&mut *tx)
        .await?;
    for org in orgs {
        org.store(&mut tx).await?;
    }
    tx.commit().await?;
    Ok(())
}
