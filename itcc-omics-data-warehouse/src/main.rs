use itcc_omics_data_warehouse::{init_tracing, run_with_config};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    info!("logger initialized");
    run_with_config().await
}
