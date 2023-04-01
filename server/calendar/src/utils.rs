use diesel::{Connection, PgConnection};

pub fn establish_connection() -> PgConnection {
    let username = std::env::var("POSTGRES_USER").unwrap_or("postgres".to_string());
    let password = std::env::var("POSTGRES_PASSWORD").unwrap_or("password".to_string());
    let url = std::env::var("POSTGRES_URL").unwrap_or("localhost".to_string());
    let db = std::env::var("POSTGRES_DB").unwrap_or(username.clone());
    let connection_string = format!("postgres://{username}:{password}@{url}/{db}");
    PgConnection::establish(&connection_string).expect("Cannot open database")
}
