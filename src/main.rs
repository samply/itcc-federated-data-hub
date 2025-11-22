use tracing_subscriber::{fmt, EnvFilter};
use omics_endpoint::run_with_config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("omics_endpoint=info".parse().unwrap()),
        )
        .init();
    run_with_config().await
}

