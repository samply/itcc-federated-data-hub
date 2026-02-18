use anyhow::anyhow;
use beam_lib::AppId;
use serde::{Deserialize, Serialize};

pub mod s3;

#[derive(Debug, Deserialize, Serialize)]
pub struct MetaData {
    pub maf_id: String,
    pub partner_id: String,
    pub checked_fhir: bool,
}

pub fn parse_beam_id(id: &str) -> Result<AppId, String> {
    let mut it = id.splitn(3, '.'); // split into 3 parts max
    let app = it.next().unwrap_or("");
    let proxy = it.next().unwrap_or("");
    let broker = it.next().unwrap_or("");
    if !app.is_empty() && !proxy.is_empty() && !broker.is_empty() {
        Ok(AppId::new_unchecked(id))
    } else {
        Err("beam-id must be <app>.<proxy>.<broker>".into())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMeta {
    #[serde(deserialize_with = "deserialize_filename")]
    pub suggested_name: Option<String>,

    pub meta: Option<serde_json::Value>,
}

pub fn deserialize_filename<'de, D: serde::Deserializer<'de>>(
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
