use omics_endpoint::run_with_config;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("omics_endpoint=info"));
    fmt().with_env_filter(filter).init();
    run_with_config().await
}
