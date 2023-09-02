use log::info;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::HashMap;

#[derive(Debug)]
struct Alias {
    alias: String,
    key: String,    // the key is the id of the entry
    r#type: String, // what we display in the url
    visible_id: String,
}

#[derive(Debug, Deserialize)]
struct AliasData {
    visible_id: Option<String>,
    aliases: Vec<String>,
    r#type: String, // what we display in the url
}
struct AliasIterator {
    alias_data: AliasData,
    key: String,
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

impl From<(String, AliasData)> for AliasIterator {
    fn from((key, alias_data): (String, AliasData)) -> Self {
        Self {
            alias_data,
            key,
            state: Default::default(),
        }
    }
}
impl Iterator for AliasIterator {
    type Item = Alias;
    fn next(&mut self) -> Option<Self::Item> {
        use AliasIteratorState as State;
        let visible_id = self
            .alias_data
            .visible_id
            .clone()
            .unwrap_or(self.key.clone());
        let alias_len = self.alias_data.aliases.len();
        let state = self.state;
        self.state = self.state.next_state();
        match state {
            State::Key => Some(Alias {
                alias: self.key.clone(),
                key: self.key.clone(),
                r#type: self.alias_data.r#type.clone(),
                visible_id,
            }),
            State::VisibleId => Some(Alias {
                alias: visible_id.clone(),
                key: self.key.clone(),
                r#type: self.alias_data.r#type.clone(),
                visible_id,
            }),
            State::Alias(index) if index < alias_len => Some(Alias {
                alias: self.alias_data.aliases[index].clone(),
                key: self.key.clone(),
                r#type: self.alias_data.r#type.clone(),
                visible_id,
            }),
            State::Alias(_) | State::Done => None,
        }
    }
}

impl Alias {
    async fn store(
        self,
        pool: &SqlitePool,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query!(
            r#"INSERT OR REPLACE INTO aliases (alias, key, type, visible_id)
            VALUES (?, ?, ?, ?)"#,
            self.alias,
            self.key,
            self.r#type,
            self.visible_id
        )
        .execute(pool)
        .await
    }
}

pub(crate) async fn load_all_to_db(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let raw_aliase = reqwest::get(format!("{cdn_url}/api_data.json"))
        .await?
        .json::<HashMap<String, AliasData>>()
        .await?;
    let set_aliase = raw_aliase
        .into_iter()
        .map(AliasIterator::from)
        .flat_map(|alias| alias.into_iter())
        .map(|alias| alias.store(pool));
    futures::future::try_join_all(set_aliase).await?;
    info!("loaded aliases");

    Ok(())
}
