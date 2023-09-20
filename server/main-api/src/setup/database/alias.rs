use log::info;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::time::Instant;

#[derive(Debug)]
struct Alias {
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
            state: Default::default(),
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
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query!(
            r#"INSERT OR REPLACE INTO aliases (alias, key, type, visible_id)
            VALUES (?, ?, ?, ?)"#,
            self.alias,
            self.key,
            self.r#type,
            self.visible_id
        )
        .execute(&mut **tx)
        .await
    }
}

pub(crate) async fn load_all_to_db(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let raw_aliase = reqwest::get(format!("{cdn_url}/api_data.json"))
        .await?
        .json::<Vec<AliasData>>()
        .await?;
    let start = Instant::now();
    let set_aliase = raw_aliase
        .into_iter()
        .map(AliasIterator::from)
        .flat_map(|alias| alias.into_iter());
    let mut tx = pool.begin().await?;
    for task in set_aliase {
        task.store(&mut tx).await?;
    }
    tx.commit().await?;
    info!("loaded aliases in {elapsed:?}", elapsed = start.elapsed());

    Ok(())
}
