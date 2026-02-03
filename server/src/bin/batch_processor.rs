use sqlx::postgres::PgPoolOptions;
use tracing::info;

fn connection_string() -> String {
    let username = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
    let password = std::env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "CHANGE_ME".to_string());
    let url = std::env::var("POSTGRES_URL").unwrap_or_else(|_| "localhost".to_string());
    let db = std::env::var("POSTGRES_DB").unwrap_or_else(|_| username.clone());
    format!("postgres://{username}:{password}@{url}/{db}")
}

fn setup_logging() {
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::fmt::Layer;
    use tracing_subscriber::prelude::*;
    
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let filter = format!(
        "{log_level},hyper=info,rustls=info,h2=info,sqlx=info,hickory_resolver=info,hickory_proto=info"
    );

    let filter = EnvFilter::builder().parse_lossy(filter);

    tracing_log::LogTracer::builder()
        .with_interest_cache(tracing_log::InterestCacheConfig::default())
        .init()
        .expect("the global logger to only be set once");

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(Layer::default().pretty());
    tracing::subscriber::set_global_default(registry).unwrap();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();
    
    info!("Starting batch processor");
    
    let pool = PgPoolOptions::new()
        .min_connections(1)
        .max_connections(5)
        .connect(&connection_string())
        .await
        .expect("Failed to connect to database");
    
    info!("Connected to database");
    
    match navigatum_server::batch_processor::process_all_batches(&pool).await {
        Ok(pr_urls) => {
            info!("Successfully processed batches. Created {} PRs", pr_urls.len());
            for (i, url) in pr_urls.iter().enumerate() {
                info!("PR {}: {}", i + 1, url);
            }
        }
        Err(e) => {
            eprintln!("Error processing batches: {:?}", e);
            std::process::exit(1);
        }
    }
    
    pool.close().await;
    info!("Batch processor completed");
    
    Ok(())
}
