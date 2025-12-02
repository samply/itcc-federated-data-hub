use omics_endpoint::run_with_config;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("omics_endpoint=info".parse().unwrap()),
        )
        .init();
    run_with_config().await
}
