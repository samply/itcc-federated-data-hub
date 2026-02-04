use crate::utils::config::AppState;
use crate::utils::error_type::ErrorType;
use anyhow::Result;
use serde::Deserialize;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct CreateSessionResp {
    #[serde(rename = "sessionId")]
    session_id: Uuid,
}

pub async fn create_session(state: &AppState) -> Result<(), ErrorType> {
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
        .map_err(|_| ErrorType::MzlSessionError)?
        .error_for_status()
        .map_err(|_| ErrorType::MzlSessionError)?
        .json::<CreateSessionResp>()
        .await
        .map_err(|_| ErrorType::MzlSessionError)?;

    debug!("sessionId = {}", session.session_id);

    Ok(())
}
