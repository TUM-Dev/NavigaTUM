use std::env;

use crate::limited::vec::LimitedVec;
use crate::setup::file_loader;
use bytes::Bytes;
use parquet::file::reader::{FileReader as _, SerializedFileReader};
use parquet::record::Field;
use sqlx::postgres::PgQueryResult;
use sqlx::{Postgres, Transaction};
use tracing::error;

#[derive(Debug, Clone)]
#[expect(
    clippy::struct_field_names,
    reason = "the `alias` field intentionally matches the type name to mirror the data model"
)]
pub(super) struct Alias {
    alias: String,
    /// the key is the id of the entry
    key: String,
    /// what we display in the url
    r#type: String,
    visible_id: String,
}

impl Alias {
    async fn store(self, tx: &mut Transaction<'_, Postgres>) -> Result<PgQueryResult, sqlx::Error> {
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
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("alias_data.parquet", &cdn_url).await?;
    let mut aliase = Vec::<Alias>::new();
    let reader = SerializedFileReader::new(Bytes::from(body))?;
    for row in reader.get_row_iter(None)? {
        let row = row?;
        let mut id = String::new();
        let mut r#type = String::new();
        let mut visible_id_opt: Option<String> = None;
        let mut nested_aliases: Vec<String> = Vec::new();
        for (col_name, field) in row.get_column_iter() {
            match (col_name.as_str(), field) {
                ("id", Field::Str(v)) => id.clone_from(v),
                ("type", Field::Str(v)) => r#type.clone_from(v),
                ("visible_id", Field::Str(v)) => visible_id_opt = Some(v.clone()),
                ("aliases", Field::ListInternal(list)) => {
                    for el in list.elements() {
                        if let Field::Str(s) = el {
                            nested_aliases.push(s.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        let visible_id = visible_id_opt.unwrap_or_else(|| id.clone());
        aliase.push(Alias {
            alias: id.clone(),
            key: id.clone(),
            r#type: r#type.clone(),
            visible_id: visible_id.clone(),
        });
        aliase.push(Alias {
            alias: visible_id.clone(),
            key: id.clone(),
            r#type: r#type.clone(),
            visible_id: visible_id.clone(),
        });
        for alias in nested_aliases {
            aliase.push(Alias {
                alias,
                key: id.clone(),
                r#type: r#type.clone(),
                visible_id: visible_id.clone(),
            });
        }
    }
    Ok(LimitedVec(aliase))
}
#[tracing::instrument(skip(tx))]
pub async fn load_all_to_db(
    aliases: LimitedVec<Alias>,
    tx: &mut Transaction<'_, Postgres>,
) -> anyhow::Result<()> {
    let mut total_errors_cnt = 0_usize;
    for task in aliases {
        if let Err(e) = task.clone().store(tx).await {
            total_errors_cnt += 1;
            if total_errors_cnt < 3 {
                error!(
                    key = task.key,
                    type = task.r#type,
                    visible_id = task.visible_id,
                    error = ?e,
                    "Could not store alias (sample {total_errors_cnt}/3)",
                );
            }
        }
    }
    if total_errors_cnt > 3 {
        error!(
            total_errors_cnt,
            "there were unreported erros in storing aliases"
        );
    }

    Ok(())
}
