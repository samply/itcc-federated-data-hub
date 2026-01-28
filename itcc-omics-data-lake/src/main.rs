use itcc_omics_data_lake::run_with_config;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("omics_endpoint=info"));
    fmt().with_env_filter(filter).init();
    tracing::info!("logger initialized");
    run_with_config().await
}
