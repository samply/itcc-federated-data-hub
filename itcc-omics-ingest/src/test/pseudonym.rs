use crate::test::{test_app_state, test_config};
use crate::utils::error_type::ErrorType;
use axum::http::StatusCode;
use beam_lib::reqwest::Url;
use itcc_omics_lib::error_type::LibError;
use itcc_omics_lib::mainzelliste::handler::{
    create_patient, create_patients, create_session, create_token, CreatePatientReq,
    CreatePatientResp, CreateTokenResp, Ids,
};
use itcc_omics_lib::mainzelliste::init_mainzelliste;
use std::collections::HashSet;
use tracing::{debug, error, info};
use uuid::Uuid;

#[ignore = "Require mainzelliste"]
#[tokio::test]
async fn test_session_pseudonym() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let uuid = create_session(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
    )
    .await?;
    Ok(())
}

#[ignore = "Require mainzelliste"]
#[tokio::test]
async fn test_token_pseudonym() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let token: CreateTokenResp = init_mainzelliste(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
        6,
    )
    .await?;
    debug!("Created token");
    debug!("{:?}", token);
    Ok(())
}

#[ignore = "Require mainzelliste"]
#[tokio::test]
async fn test_create_patient() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let token: CreateTokenResp = init_mainzelliste(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
        1,
    )
    .await?;
    info!("Created Token");
    let psy = create_patient(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
        &token.id,
        "P0KRKM80V",
    )
    .await?;
    debug!("Created patient");
    debug!("{:?}", psy);
    Ok(())
}

#[ignore = "Require mainzelliste"]
#[tokio::test]
async fn test_create_patients() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let patient_ids: HashSet<String> = [
        "patient-001",
        "patient-002",
        "patient-003",
        "patient-004",
        "patient-005",
        "patient-006",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect();
    let token: CreateTokenResp = init_mainzelliste(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
        6,
    )
    .await?;
    let psy = create_patients(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
        &token.id,
        patient_ids,
    )
    .await?;
    debug!("Created patients");
    debug!("{:?}", psy);
    Ok(())
}

#[cfg(test)]
pub async fn create_patient_debug(
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

    let res = http_client
        .post(patient_url)
        .query(&[("tokenId", token)])
        .header("mainzellisteApiKey", ml_api_key)
        .header("mainzellisteApiVersion", "3.3")
        .form(&[("localid", patient_id)])
        .send()
        .await
        .map_err(|_| LibError::MLCreatePatientError)?;

    let status = res.status();
    if status == StatusCode::NOT_FOUND {
        return Err(LibError::MLCreatePatientError);
    }
    debug!("Status code: {}", status);
    debug!("{:#?}", res);

    let pseudo = res.json::<CreatePatientResp>().await.map_err(|e| {
        error!("Failed to get patient: {}", patient_id);
        error!("Error: {e}");
        LibError::MLCreatePatientError
    })?;
    debug!("pseudo = {:?}", pseudo);
    Ok(pseudo)
}
