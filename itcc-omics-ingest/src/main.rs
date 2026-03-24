use itcc_omics_ingest::{init_tracing, run_with_config};

#[tokio::main]
async fn main() {
    init_tracing();
    run_with_config().await
}
