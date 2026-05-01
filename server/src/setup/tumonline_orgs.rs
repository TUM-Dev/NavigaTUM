use std::env;

use crate::setup::file_loader;
use bytes::Bytes;
use parquet::file::reader::{FileReader as _, SerializedFileReader};
use parquet::record::Field;
use sqlx::postgres::PgQueryResult;
use sqlx::{PgPool, Postgres, Transaction};

#[derive(Default, Debug)]
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
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<PgQueryResult, sqlx::Error> {
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
pub async fn setup(pool: &PgPool) -> anyhow::Result<()> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("tumonline_orgs.parquet", &cdn_url).await?;

    let reader = SerializedFileReader::new(Bytes::from(body))?;
    let mut orgs = Vec::new();
    for row in reader.get_row_iter(None)? {
        let row = row?;
        let mut org = DBOrg::default();
        for (col_name, field) in row.get_column_iter() {
            match (col_name.as_str(), field) {
                ("org_id", Field::Int(v)) => org.org_id = *v,
                ("code", Field::Str(v)) => org.code.clone_from(v),
                ("name_de", Field::Str(v)) => org.name_de.clone_from(v),
                ("name_en", Field::Str(v)) => org.name_en.clone_from(v),
                ("path_de", Field::Str(v)) => org.path_de = Some(v.clone()),
                ("path_en", Field::Str(v)) => org.path_en = Some(v.clone()),
                _ => {}
            }
        }
        orgs.push(org);
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
