use anyhow::anyhow;
use beam_lib::reqwest::Url;
use beam_lib::AppId;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub api_key: String,
    pub zstd_level: i32,
    pub required_omics_columns: Vec<String>,
    pub data_lake_id: AppId,
}

impl From<&Config> for AppState {
    fn from(c: &Config) -> Self {
        Self {
            api_key: c.api_key.clone(),
            zstd_level: c.zstd_level,
            required_omics_columns: c.required_omics_columns.clone(),
            data_lake_id: c.data_lake_id.clone(),
        }
    }
}

#[derive(Debug, Parser, Clone)]
pub struct Config {
    #[clap(long, env)]
    pub api_key: String,
    /// Url of the local beam proxy which is required to have sockets enabled
    #[clap(env, long, default_value = "http://beam-proxy:8081")]
    pub beam_url: Url,
    #[clap(long, env, default_value = "http://host.docker.internal:8081")]
    pub blaze_url: Url,
    #[clap(long, env, default_value = "http://host.docker.internal:7878")]
    pub mainzelliste_url: Url,
    /// Beam api key
    #[clap(env, long)]
    pub beam_secret: String,
    /// The app id of this application
    #[clap(long, env, value_parser = parse_beam_id)]
    pub beam_id: AppId,
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

fn parse_beam_id(id: &str) -> Result<AppId, String> {
    match id.split('.').collect::<Vec<_>>().as_slice() {
        [app, proxy, broker] if !app.is_empty() && !proxy.is_empty() && !broker.is_empty() => {
            Ok(AppId::new_unchecked(id))
        }
        _ => Err("beam-id must be <app>.<proxy>.<broker>".into()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMeta {
    #[serde(deserialize_with = "deserialize_filename")]
    pub suggested_name: Option<String>,

    pub meta: Option<serde_json::Value>,
}

fn deserialize_filename<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let s = Option::<String>::deserialize(deserializer)?;
    if let Some(ref f) = s {
        validate_filename(f).map_err(serde::de::Error::custom)?;
    }
    Ok(s)
}

pub fn validate_filename(name: &str) -> anyhow::Result<&str> {
    if name
        .chars()
        .all(|c| c.is_alphanumeric() || ['_', '.', '-'].contains(&c))
    {
        Ok(name)
    } else {
        Err(anyhow!("Invalid filename: {name}"))
    }
}
