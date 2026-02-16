pub mod beam;
pub mod data;
pub mod s3;
pub mod utils;

use crate::beam::run_socket_polling;
use crate::s3::get_object;
use crate::utils::config::Config;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::Client;
use beam_lib::BeamClient;
use clap::Parser;
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;
use tracing::info;

pub static DATALAKE_CONFIG: once_cell::sync::Lazy<Config> =
    once_cell::sync::Lazy::new(|| Config::parse());
pub static BEAM_CLIENT: Lazy<BeamClient> = Lazy::new(|| {
    BeamClient::new(
        &DATALAKE_CONFIG.beam_id,
        &DATALAKE_CONFIG.beam_secret,
        DATALAKE_CONFIG.beam_url.clone(),
    )
});

pub static S3_CLIENT: Lazy<OnceCell<Client>> = Lazy::new(OnceCell::const_new);

pub async fn s3_client() -> &'static Client {
    S3_CLIENT
        .get_or_init(|| async {
            info!("Initializing S3 client");
            let cfg = aws_config::defaults(BehaviorVersion::latest())
                .region(Region::new(DATALAKE_CONFIG.s3_default_region.clone()))
                .credentials_provider(Credentials::new(
                    DATALAKE_CONFIG.s3_access_key_id.clone(),
                    DATALAKE_CONFIG.s3_secret_access_key.clone(),
                    None,
                    None,
                    "static",
                ))
                .load()
                .await;
            let s3_conifg = aws_sdk_s3::config::Builder::from(&cfg)
                .endpoint_url(DATALAKE_CONFIG.s3_endpoint_url.clone())
                .force_path_style(true)
                .build();
            Client::from_conf(s3_conifg)
        })
        .await
}
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
    tokio::select! {
        res = run_socket_polling() => res?,
        _ = tokio::signal::ctrl_c() => tracing::info!("Shutting down"),
    }
    Ok(())
}
