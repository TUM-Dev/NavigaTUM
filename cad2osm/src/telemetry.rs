pub fn setup_logging(verbosity: u8) {
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::fmt::Layer;
    use tracing_subscriber::prelude::*;
    let default_level = if cfg!(any(debug_assertions, test)) {
        match verbosity {
            0 => "debug",
            _ => "trace",
        }
    } else {
        match verbosity {
            0 => "info",
            1 => "debug",
            _ => "trace",
        }
    };

    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| default_level.to_string());
    // these overrides exist to filter away stuff I don't think we should ever care about
    let filter = format!("{log_level},hyper=info,rustls=info,h2=info,sqlx=info,hickory_resolver=info,hickory_proto=info");

    let filter = EnvFilter::builder().parse_lossy(filter);

    tracing_log::LogTracer::builder()
        .with_interest_cache(tracing_log::InterestCacheConfig::default())
        .init()
        .expect("the global logger to only be set once");

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(cfg!(not(any(debug_assertions, test))).then(|| Layer::default().json()))
        .with(cfg!(any(debug_assertions, test)).then(|| Layer::default().pretty()));
    tracing::subscriber::set_global_default(registry).unwrap();
}
