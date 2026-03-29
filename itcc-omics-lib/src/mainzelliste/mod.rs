use crate::error_type::LibError;
use crate::mainzelliste::handler::{
    create_patients, create_session, create_token, extract_mapping, CreatePatientResp,
    CreateTokenResp,
};
use beam_lib::reqwest::Url;
use std::collections::{HashMap, HashSet};
use tracing::debug;
use uuid::Uuid;

pub mod handler;

pub async fn init_mainzelliste(
    http_client: &reqwest::Client,
    ml_api_key: &str,
    ml_url: &Url,
    allowed_uses: usize,
) -> Result<CreateTokenResp, LibError> {
    let session_id = create_session(http_client, ml_api_key, ml_url).await?;
    Ok(create_token(http_client, ml_api_key, ml_url, &session_id, allowed_uses).await?)
}

pub async fn encryption_ml(
    http_client: &reqwest::Client,
    ml_api_key: &str,
    ml_url: &Url,
    token: &Uuid,
    patient_ids: HashSet<String>,
) -> Result<HashMap<String, String>, LibError> {
    let pseudonym_res: Vec<CreatePatientResp> =
        create_patients(http_client, ml_api_key, ml_url, token, patient_ids).await?;
    let local_crypto_ids: HashMap<String, String> = extract_mapping(pseudonym_res)?;
    debug!("Mapping: {:#?}", local_crypto_ids);
    Ok(local_crypto_ids)
}
