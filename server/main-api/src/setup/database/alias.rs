use std::time::Instant;

use serde::Deserialize;
use tracing::debug;

#[derive(Debug)]
pub(super) struct Alias {
    alias: String,
    key: String,    // the key is the id of the entry
    r#type: String, // what we display in the url
    visible_id: String,
}

#[derive(Debug, Deserialize)]
struct AliasData {
    id: String,
    visible_id: Option<String>,
    aliases: Vec<String>,
    r#type: String, // what we display in the url
}
struct AliasIterator {
    data: AliasData,
    state: AliasIteratorState,
}
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
enum AliasIteratorState {
    #[default]
    Key,
    VisibleId,
    Alias(usize),
    Done,
}
impl AliasIteratorState {
    fn next_state(&mut self) -> Self {
        match self {
            Self::Key => Self::VisibleId,
            Self::VisibleId => Self::Alias(0),
            Self::Alias(i) => Self::Alias(*i + 1),
            Self::Done => Self::Done,
        }
    }
}

impl From<AliasData> for AliasIterator {
    fn from(alias_data: AliasData) -> Self {
        Self {
            data: alias_data,
            state: AliasIteratorState::default(),
        }
    }
}
impl Iterator for AliasIterator {
    type Item = Alias;
    fn next(&mut self) -> Option<Self::Item> {
        use AliasIteratorState as State;
        let visible_id = self.data.visible_id.clone().unwrap_or(self.data.id.clone());
        let alias_len = self.data.aliases.len();
        let state = self.state;
        self.state = self.state.next_state();
        match state {
            State::Key => Some(Alias {
                alias: self.data.id.clone(),
                key: self.data.id.clone(),
                r#type: self.data.r#type.clone(),
                visible_id,
            }),
            State::VisibleId => Some(Alias {
                alias: visible_id.clone(),
                key: self.data.id.clone(),
                r#type: self.data.r#type.clone(),
                visible_id,
            }),
            State::Alias(index) if index < alias_len => Some(Alias {
                alias: self.data.aliases[index].clone(),
                key: self.data.id.clone(),
                r#type: self.data.r#type.clone(),
                visible_id,
            }),
            State::Alias(_) | State::Done => None,
        }
    }
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
pub async fn download_updates(
    keys_which_need_updating: &[String],
) -> Result<Vec<Alias>, crate::BoxedError> {
    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    Ok(reqwest::get(format!("{cdn_url}/api_data.json"))
        .await?
        .json::<Vec<AliasData>>()
        .await?
        .into_iter()
        .filter(|d| keys_which_need_updating.is_empty() || keys_which_need_updating.contains(&d.id))
        .map(AliasIterator::from)
        .flat_map(IntoIterator::into_iter)
        .collect::<Vec<Alias>>())
}
pub async fn load_all_to_db(
    aliases: Vec<Alias>,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), crate::BoxedError> {
    let start = Instant::now();
    for task in aliases {
        task.store(tx).await?;
    }
    debug!("loaded aliases in {elapsed:?}", elapsed = start.elapsed());

    Ok(())
}
