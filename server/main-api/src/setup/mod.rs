mod database;

#[cfg(skip_meilisearch)]
mod meilisearch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    #[cfg(skip_meilisearch)]
    meilisearch::setup_meilisearch().await?;
    database::setup_database().await?;
    Ok(())
}
