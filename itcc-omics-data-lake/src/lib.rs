pub mod beam;
pub mod s3;
pub mod utils;

use crate::beam::run_socket_polling;
use crate::utils::config::Config;
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use beam_lib::BeamClient;
use clap::Parser;
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;
use tracing::info;

pub static CONFIG: Lazy<Config> = Lazy::new(Config::parse);
pub static BEAM_CLIENT: Lazy<BeamClient> = Lazy::new(|| {
    BeamClient::new(
        &CONFIG.beam_id,
        &CONFIG.beam_secret,
        CONFIG.beam_url.clone(),
    )
});

pub static S3_CLIENT: Lazy<OnceCell<Client>> = Lazy::new(OnceCell::const_new);

pub async fn s3_client() -> &'static Client {
    S3_CLIENT
        .get_or_init(|| async {
            info!("Initializing S3 client");
            let cfg = aws_config::defaults(BehaviorVersion::latest())
                .region(Region::new(CONFIG.s3_default_region.clone()))
                .credentials_provider(Credentials::new(
                    CONFIG.s3_access_key_id.clone(),
                    CONFIG.s3_secret_access_key.clone(),
                    None,
                    None,
                    "static",
                ))
                .load()
                .await;
            let s3_conifg = aws_sdk_s3::config::Builder::from(&cfg)
                .endpoint_url(CONFIG.s3_endpoint_url.clone())
                .force_path_style(true)
                .build();
            Client::from_conf(s3_conifg)
        })
        .await
}
pub async fn run_with_config() -> anyhow::Result<()> {
    info!(
        "S3_ACCESS_KEY_ID='{}' len={}",
        CONFIG.s3_access_key_id,
        CONFIG.s3_access_key_id.len()
    );
    info!(
        "S3_SECRET_ACCESS_KEY len={}",
        CONFIG.s3_secret_access_key.len()
    );
    tokio::select! {
        res = run_socket_polling() => res?,
        _ = tokio::signal::ctrl_c() => tracing::info!("Shutting down"),
    }
    Ok(())
}
