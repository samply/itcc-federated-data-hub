pub mod beam;
pub mod cbio_portal;
pub mod data;
pub mod utils;

use crate::beam::{run_socket_polling, run_task_polling};
use crate::utils::config::Config;
use beam_lib::BeamClient;
use clap::Parser;
use itcc_omics_lib::s3::client::{init_s3_client, ConfigS3};
use once_cell::sync::Lazy;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

pub static DATALAKE_CONFIG: once_cell::sync::Lazy<Config> =
    once_cell::sync::Lazy::new(|| Config::parse());
pub static BEAM_TASK_CLIENT: Lazy<BeamClient> = Lazy::new(|| {
    BeamClient::new(
        &DATALAKE_CONFIG.beam_task_id,
        &DATALAKE_CONFIG.beam_secret,
        DATALAKE_CONFIG.beam_url.clone(),
    )
});

pub static BEAM_SOCKET_CLIENT: Lazy<BeamClient> = Lazy::new(|| {
    BeamClient::new(
        &DATALAKE_CONFIG.beam_socket_id,
        &DATALAKE_CONFIG.beam_secret,
        DATALAKE_CONFIG.beam_url.clone(),
    )
});

pub async fn run_with_config() -> anyhow::Result<()> {
    info!(
        "S3_ACCESS_KEY_ID='{}' len={}",
        DATALAKE_CONFIG.s3_access_key_id,
        DATALAKE_CONFIG.s3_access_key_id.len()
    );
    info!(
        "S3_SECRET_ACCESS_KEY len={}",
        DATALAKE_CONFIG.s3_secret_access_key.len()
    );
    info!("Beam Socket init: {:?}", BEAM_SOCKET_CLIENT);
    info!("Beam Task init: {:?}", BEAM_TASK_CLIENT);
    let custom_config: ConfigS3 = ConfigS3 {
        s3_default_region: DATALAKE_CONFIG.s3_default_region.clone(),
        s3_access_key_id: DATALAKE_CONFIG.s3_access_key_id.clone(),
        s3_secret_access_key: DATALAKE_CONFIG.s3_secret_access_key.clone(),
        s3_endpoint_url: DATALAKE_CONFIG.s3_endpoint_url.to_string(),
    };
    init_s3_client(custom_config).await;
    run_polling().await
}

async fn run_polling() -> anyhow::Result<()> {
    // polling sockets or tasks
    if DATALAKE_CONFIG.enable_sockets {
        tokio::select! {
        res = run_socket_polling() => res?,
        _ = tokio::signal::ctrl_c() => info!("Shutting down"),
        }
    } else {
        tokio::select! {
        res = run_task_polling() => res?,
        _ = tokio::signal::ctrl_c() => info!("Shutting down"),
        }
    }
    Ok(())
}

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("itcc_omics_data_warehouse=debug,tower_http=debug,info")
        }))
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .pretty(),
            //.json(),
        )
        .init();
}
