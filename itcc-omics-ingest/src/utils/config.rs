use beam_lib::reqwest::Url as beam_Url;
use beam_lib::{AppId, BeamClient};
use clap::Parser;
use itcc_omics_lib::beam::parse_beam_id;
use reqwest::{Client, Url};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
    pub beam_client: BeamClient,
    pub api_key: Arc<str>,
    pub zstd_level: i32,
    pub required_omics_columns: Arc<[String]>,
    pub partner_id: Arc<str>,
    pub services: Arc<Services>,
}
#[derive(Clone)]
pub struct Services {
    pub ml_url: Url,
    pub ml_api_key: Arc<str>,
    pub blaze_url: Url,
    pub beam_url: beam_Url,
    pub beam_id: AppId,
    pub beam_secret: Arc<str>,
    pub enable_sockets: bool,
    pub dwh_socket_id: AppId,
    pub dwh_task_id: AppId,
}

impl From<&IngestConfig> for AppState {
    fn from(c: &IngestConfig) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client build failed");
        let beam_client = BeamClient::new(&c.beam_id, c.beam_secret.as_str(), c.beam_url.clone());
        Self {
            http,
            beam_client,
            api_key: Arc::from(c.api_key.as_str()),
            zstd_level: c.zstd_level,
            required_omics_columns: Arc::from(c.required_omics_columns.as_slice()),
            partner_id: Arc::from(c.partner_id.as_str()),
            services: Arc::from(Services {
                ml_url: c.ml_url.clone(),
                ml_api_key: Arc::from(c.ml_api_key.as_str()),
                blaze_url: c.blaze_url.clone(),
                beam_url: c.beam_url.clone(),
                beam_id: c.beam_id.clone(),
                beam_secret: Arc::from(c.beam_secret.as_str()),
                enable_sockets: c.enable_sockets,
                dwh_socket_id: c.dwh_socket_id.clone(),
                dwh_task_id: c.dwh_task_id.clone(),
            }),
        }
    }
}

#[derive(Debug, Parser, Clone)]
pub struct IngestConfig {
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
    /// The app id of the central data warehouse(receiver)
    #[clap(long, env, value_parser = parse_beam_id)]
    pub dwh_task_id: AppId,
    #[clap(long, env, value_parser = parse_beam_id)]
    pub dwh_socket_id: AppId,
    #[clap(long, env, default_value = "3")]
    pub zstd_level: i32,
    #[clap(
        long,
        env,
        value_delimiter = ',',
        default_value = "Hugo_Symbol,NCBI_Build,Chromosome,Start_Position,End_Position,Variant_Classification,Variant_Type,Reference_Allele,Tumor_Seq_Allele1,Tumor_Seq_Allele2,Tumor_Sample_Barcode"
    )]
    pub required_omics_columns: Vec<String>,
}
