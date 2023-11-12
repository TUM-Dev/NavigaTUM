pub fn connection_string() -> String {
    let username = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
    let password = std::env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "password".to_string());
    let url = std::env::var("POSTGRES_URL").unwrap_or_else(|_| "localhost".to_string());
    let db = std::env::var("POSTGRES_DB").unwrap_or_else(|_| username.clone());
    format!("postgres://{username}:{password}@{url}/{db}")
}
