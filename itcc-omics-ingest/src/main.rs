use itcc_omics_ingest::run_with_config;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("itcc_omics_ingest=info"));
    fmt().with_env_filter(filter).init();
    run_with_config().await
}
