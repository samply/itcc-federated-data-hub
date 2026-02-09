use crate::utils::config::AppState;
use crate::utils::error_type::ErrorType;
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct CreateSessionResp {
    #[serde(rename = "sessionId")]
    session_id: Uuid,
}

pub async fn create_session(state: &AppState) -> Result<Uuid, ErrorType> {
    let url_mzl = state
        .services
        .ml_url
        .join("/sessions")
        .expect("mainzelliste url should be present");
    let session: CreateSessionResp = state
        .http
        .post(url_mzl)
        .header("mainzellisteApiKey", &state.services.ml_api_key)
        .send()
        .await
        .map_err(|_| ErrorType::MlSessionError)?
        .error_for_status()
        .map_err(|_| ErrorType::MlSessionError)?
        .json::<CreateSessionResp>()
        .await
        .map_err(|_| ErrorType::MlSessionError)?;

    debug!("sessionId = {}", session.session_id);

    Ok(session.session_id)
}

#[derive(Debug, Serialize)]
pub struct CreateTokenReq {
    #[serde(rename = "type")]
    token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "allowedUses")]
    allowed_uses: Option<u32>,
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
    state: &AppState,
    session_id: &Uuid,
    allowed_uses: u32,
) -> Result<CreateTokenResp, ErrorType> {
    let token_req = CreateTokenReq {
        token_type: "addPatient".to_string(),
        allowed_uses: Some(allowed_uses),
        data: TokenData {
            idtypes: vec!["localid".to_string(), "cryptoid".to_string()],
        },
    };
    let token_url = state
        .services
        .ml_url
        .join(&format!("/sessions/{session_id}/tokens"))
        .expect("mainzelliste url should be present");

    let token: CreateTokenResp = state
        .http
        .post(token_url)
        .header("mainzellisteApiKey", &state.services.ml_api_key)
        .json(&token_req)
        .send()
        .await
        .map_err(|_| ErrorType::MlTokenError)?
        .error_for_status()
        .map_err(|_| ErrorType::MlTokenError)?
        .json::<CreateTokenResp>()
        .await
        .map_err(|_| ErrorType::MlTokenError)?;

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
    state: &AppState,
    token: &Uuid,
    patient_id: &str,
) -> Result<(), ErrorType> {
    let patient_id = CreatePatientReq {
        ids: Ids {
            localid: patient_id.to_string(),
        },
    };
    let patient_url = state
        .services
        .ml_url
        .join("patients")
        .expect("mainzelliste url should be present");

    let pseudo = state
        .http
        .post(patient_url)
        .query(&["tokenId", token.to_string().as_str()])
        .header("mainzellisteApiKey", &state.services.ml_api_key)
        .header("mainzellisteApiVersion", "2.0")
        .json(&patient_id)
        .send()
        .await
        .map_err(|_| ErrorType::MLCreatePatientError)?
        .error_for_status()
        .map_err(|_| ErrorType::MLCreatePatientError)?
        .json::<CreatePatientResp>()
        .await
        .map_err(|_| ErrorType::MLCreatePatientError)?;
    Ok(())
}
