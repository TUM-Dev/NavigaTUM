pub mod statistics;

use diesel::{Connection, SqliteConnection};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LangQueryArgs {
    lang: Option<String>,
}

impl LangQueryArgs {
    pub fn should_use_english(&self) -> bool {
        self.lang.as_ref().map_or(false, |c| c == "en")
    }
}

pub fn establish_connection() -> SqliteConnection {
    let database_loc = std::env::var("DB_LOCATION").unwrap_or("data/api_data.db".to_string());
    SqliteConnection::establish(&database_loc).expect("Cannot open database")
}
