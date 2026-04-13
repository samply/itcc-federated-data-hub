use crate::error_type::LibError;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct CreateSessionResp {
    #[serde(rename = "sessionId")]
    session_id: Uuid,
}

pub async fn create_session(
    http_client: &reqwest::Client,
    ml_api_key: &str,
    ml_url: &Url,
) -> Result<Uuid, LibError> {
    let url_mzl = ml_url
        .join("/sessions")
        .expect("mainzelliste url should be present");
    let session: CreateSessionResp = http_client
        .post(url_mzl)
        .header("mainzellisteApiKey", ml_api_key)
        .send()
        .await
        .map_err(|_| LibError::MlSessionError)?
        .error_for_status()
        .map_err(|_| LibError::MlSessionError)?
        .json::<CreateSessionResp>()
        .await
        .map_err(|_| LibError::MlSessionError)?;

    debug!("sessionId = {}", session.session_id);

    Ok(session.session_id)
}

#[derive(Debug, Serialize)]
pub struct CreateTokenReq {
    #[serde(rename = "type")]
    token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "allowedUses")]
    allowed_uses: Option<usize>,
    data: TokenData,
}

#[derive(Debug, Serialize)]
pub struct TokenData {
    // Some Mainzelliste versions use idtypes vs idTypes.
    // We'll send "idtypes" here; if your server rejects it, change rename to "idTypes".
    #[serde(rename = "idtypes")]
    idtypes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTokenResp {
    #[serde(rename = "tokenId")]
    pub id: Uuid,
    pub uri: String,
}

pub async fn create_token(
    http_client: &reqwest::Client,
    ml_api_key: &str,
    ml_url: &Url,
    session_id: &Uuid,
    allowed_uses: usize,
) -> Result<CreateTokenResp, LibError> {
    let token_req = CreateTokenReq {
        token_type: "addPatient".to_string(),
        allowed_uses: Some(allowed_uses),
        data: TokenData {
            idtypes: vec!["localid".to_string(), "cryptoid".to_string()],
        },
    };
    let token_url = ml_url
        .join(&format!("/sessions/{session_id}/tokens"))
        .expect("mainzelliste url should be present");

    let token: CreateTokenResp = http_client
        .post(token_url)
        .header("mainzellisteApiKey", ml_api_key)
        .json(&token_req)
        .send()
        .await
        .map_err(|_| LibError::MlTokenError)?
        .error_for_status()
        .map_err(|_| LibError::MlTokenError)?
        .json::<CreateTokenResp>()
        .await
        .map_err(|_| LibError::MlTokenError)?;

    debug!("tokenId = {}", token.id);
    Ok(token)
}
#[derive(Debug, Serialize)]
pub struct CreatePatientReq {
    pub ids: Ids,
}
#[derive(Debug, Serialize)]
pub struct Ids {
    pub localid: String,
}

pub type CreatePatientResp = Vec<TypeId>;

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeId {
    #[serde(rename = "idType")]
    pub id_type: String,
    #[serde(rename = "idString")]
    pub id_string: String,
    pub tentative: bool,
    pub uri: String,
}

pub async fn create_patient(
    http_client: &reqwest::Client,
    ml_api_key: &str,
    ml_url: &Url,
    token: &Uuid,
    patient_id: &str,
) -> Result<CreatePatientResp, LibError> {
    let body = CreatePatientReq {
        ids: Ids {
            localid: patient_id.to_string(),
        },
    };
    let patient_url = ml_url
        .join("patients")
        .expect("mainzelliste url should be present");

    let pseudo: CreatePatientResp = http_client
        .post(patient_url)
        .query(&[("tokenId", token)])
        .header("mainzellisteApiKey", ml_api_key)
        .header("mainzellisteApiVersion", "3.3")
        .json(&body)
        .send()
        .await
        .map_err(|_| LibError::MLCreatePatientError)?
        .error_for_status()
        .map_err(|_| LibError::MLCreatePatientError)?
        .json::<CreatePatientResp>()
        .await
        .map_err(|_| LibError::MLCreatePatientError)?;
    debug!("pseudo = {:?}", pseudo);
    Ok(pseudo)
}

pub async fn create_patients(
    http_client: &reqwest::Client,
    ml_api_key: &str,
    ml_url: &Url,
    token: &Uuid,
    patient_ids: HashSet<String>,
) -> Result<Vec<CreatePatientResp>, LibError> {
    let mut out = Vec::with_capacity(patient_ids.len());

    for pid in patient_ids {
        let resp: CreatePatientResp =
            create_patient(http_client, ml_api_key, ml_url, token, pid.as_str()).await?;
        out.push(resp);
    }

    Ok(out)
}

pub fn extract_mapping(resp: Vec<CreatePatientResp>) -> Result<HashMap<String, String>, LibError> {
    resp.into_iter()
        .map(|r| {
            let local = r
                .iter()
                .find(|x| x.id_type == "localid")
                .map(|x| x.id_string.clone())
                .ok_or(LibError::PseudoError)?;

            let crypto = r
                .iter()
                .find(|x| x.id_type == "cryptoid")
                .map(|x| x.id_string.clone())
                .ok_or(LibError::PseudoError)?;

            Ok((local, crypto))
        })
        .collect::<Result<HashMap<String, String>, LibError>>()
}
