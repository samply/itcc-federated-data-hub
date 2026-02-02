use anyhow::anyhow;
use beam_lib::reqwest::Url;
use beam_lib::AppId;
use clap::Parser;
use serde::{Deserialize, Serialize};

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
        .all(|c| c.is_alphanumeric() || ['_', '.', '-', '/'].contains(&c))
    {
        Ok(name)
    } else {
        Err(anyhow!("Invalid filename: {name}"))
    }
}
