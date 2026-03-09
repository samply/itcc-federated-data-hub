use beam_lib::reqwest::Url as beam_Url;
use beam_lib::{AppId, BeamClient};
use clap::Parser;
use itcc_omics_lib::beam::parse_beam_id;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
    pub beam_client: BeamClient,
    pub api_key: String,
    pub zstd_level: i32,
    pub required_omics_columns: Vec<String>,
    pub data_lake_id: AppId,
    pub partner_id: String,
    pub services: Services,
}
#[derive(Clone)]
pub struct Services {
    pub ml_url: Url,
    pub ml_api_key: String,
    pub blaze_url: Url,
    pub beam_url: beam_Url,
    pub beam_id: AppId,
    pub beam_secret: String,
    pub enable_sockets: bool,
}

impl From<&Config> for AppState {
    fn from(c: &Config) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client build failed");
        let beam_client = BeamClient::new(&c.beam_id, c.beam_secret.as_str(), c.beam_url.clone());
        Self {
            http,
            beam_client,
            api_key: c.api_key.clone(),
            zstd_level: c.zstd_level,
            required_omics_columns: c.required_omics_columns.clone(),
            data_lake_id: c.data_lake_id.clone(),
            partner_id: c.partner_id.clone(),
            services: Services {
                ml_url: c.ml_url.clone(),
                ml_api_key: c.ml_api_key.clone(),
                blaze_url: c.blaze_url.clone(),
                beam_url: c.beam_url.clone(),
                beam_id: c.beam_id.clone(),
                beam_secret: c.beam_secret.to_string(),
                enable_sockets: c.enable_sockets,
            },
        }
    }
}

#[derive(Debug, Parser, Clone)]
pub struct Config {
    #[clap(long, env)]
    pub api_key: String,
    /// Url of the local beam proxy which is required to have sockets enabled
    #[clap(env, long, default_value = "http://beam-proxy:8081")]
    pub beam_url: beam_Url,
    #[clap(env, long, default_value = "itcc-inform")]
    pub partner_id: String,
    #[clap(long, env, default_value = "http://blaze:8080")]
    pub blaze_url: Url,
    #[clap(long, env, default_value = "http://mainzelliste:8080")]
    pub ml_url: Url,
    #[clap(long, env)]
    pub ml_api_key: String,
    /// Beam api key
    #[clap(env, long)]
    pub beam_secret: String,
    /// The app id of this application
    #[clap(long, env, value_parser = parse_beam_id)]
    pub beam_id: AppId,
    #[clap(env, long, default_value_t = false)]
    pub enable_sockets: bool,
    /// The app id of the central data lake(receiver)
    #[clap(long, env, value_parser = parse_beam_id)]
    pub data_lake_id: AppId,
    #[clap(long, env, default_value = "3")]
    pub zstd_level: i32,
    #[clap(
        long,
        env,
        value_delimiter = ',',
        default_value = "Hugo_Symbol,Chromosome,Start_Position,End_Position,Variant_Classification,Variant_Type,Reference_Allele,Tumor_Seq_Allele1,Tumor_Seq_Allele2,Tumor_Sample_Barcode"
    )]
    pub required_omics_columns: Vec<String>,
}
