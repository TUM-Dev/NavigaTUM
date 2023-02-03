use diesel::{Connection, SqliteConnection};

pub mod statistics;

pub fn establish_connection() -> SqliteConnection {
    let database_loc = std::env::var("DB_LOCATION").unwrap_or("data/calendar_data.db".to_string());
    SqliteConnection::establish(&database_loc).expect("Cannot open database")
}
