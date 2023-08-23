mod meilisearch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    meilisearch::setup_meilisearch().await?;
    Ok(())
}
