use crate::utils::config::AppState;
use crate::CLIENT;
use anyhow::{anyhow, Context, Result};
use reqwest::Url;
use serde::Deserialize;
use tracing::debug;
use uuid::Uuid;
use crate::utils::error_type::ErrorType;

#[derive(Debug, Deserialize)]
struct CreateSessionResp {
    #[serde(rename = "sessionId")]
    session_id: Uuid,
}

pub async fn create_session(mzl_base_url: &Url, api_mzl_key: &str) -> Result<(), ErrorType> {
    let url_mzl = mzl_base_url.join("/sessions").expect("mainzelliste url should be present");
    let session: CreateSessionResp = (&*CLIENT)
        .post(url_mzl)
        .header("mainzellisteApiKey", api_mzl_key)
        .send()
        .await
        .map_err(|_|ErrorType::MzlSessionError)?
        .error_for_status()
        .map_err(|_|ErrorType::MzlSessionError)?
        .json::<CreateSessionResp>()
        .await
        .map_err(|_|ErrorType::MzlSessionError)?;

    debug!("sessionId = {}", session.session_id);

    Ok(())
}
