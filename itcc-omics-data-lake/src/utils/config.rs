use beam_lib::reqwest::Url;
use beam_lib::AppId;
use clap::Parser;
use itcc_omics_lib::*;

#[derive(Debug, Parser, Clone)]
pub struct Config {
    /// Url of the local beam proxy which is required to have sockets enabled
    #[clap(env, long, default_value = "http://beam-proxy:8081")]
    pub beam_url: Url,
    /// Beam api key
    #[clap(env, long)]
    pub beam_secret: String,
    /// The app id of this application
    #[clap(long, env, value_parser = parse_beam_id)]
    pub beam_id: AppId,
    #[clap(env, long, default_value = "http://garage:3900")]
    pub s3_endpoint_url: Url,
    #[clap(env, long)]
    pub s3_access_key_id: String,
    #[clap(env, long)]
    pub s3_secret_access_key: String,
    #[clap(env, long)]
    pub s3_default_region: String,
    #[clap(env, long, default_value = "omics")]
    pub s3_bucket: String,
}
