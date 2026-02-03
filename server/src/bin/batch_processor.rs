use tracing::info;

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
    
    // Finalize the current batch PR (remove in-progress label)
    match navigatum_server::batch_processor::finalize_batch_pr().await {
        Ok(()) => {
            info!("Successfully finalized batch PR");
        }
        Err(e) => {
            eprintln!("Error finalizing batch PR: {:?}", e);
            std::process::exit(1);
        }
    }
    
    info!("Batch processor completed");
    
    Ok(())
}
