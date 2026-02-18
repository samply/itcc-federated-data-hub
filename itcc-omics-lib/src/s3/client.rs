use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::Client;
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;
use tracing::info;

pub struct ConfigS3 {
    pub s3_default_region: String,
    pub s3_access_key_id: String,
    pub s3_secret_access_key: String,
    pub s3_endpoint_url: String,
}

pub static S3_CLIENT: Lazy<OnceCell<Client>> = Lazy::new(OnceCell::const_new);
pub async fn init_s3_client(custom_conf: ConfigS3) -> &'static Client {
    S3_CLIENT
        .get_or_init(|| async move {
            let cfg = aws_config::defaults(BehaviorVersion::latest())
                .region(Region::new(custom_conf.s3_default_region))
                .credentials_provider(Credentials::new(
                    custom_conf.s3_access_key_id,
                    custom_conf.s3_secret_access_key,
                    None,
                    None,
                    "static",
                ))
                .load()
                .await;

            let s3_config = aws_sdk_s3::config::Builder::from(&cfg)
                .endpoint_url(custom_conf.s3_endpoint_url)
                .force_path_style(true)
                .build();

            Client::from_conf(s3_config)
        })
        .await
}

pub async fn s3_client() -> &'static Client {
    S3_CLIENT.get().expect("S3 client not initialized")
}
