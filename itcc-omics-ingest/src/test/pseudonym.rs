use crate::pseudonym::handler::{
    create_patient, create_patients, create_session, create_token, CreateTokenResp,
};
use crate::test::{test_app_state, test_config};
use crate::utils::error_type::ErrorType;

#[ignore = "Require mainzelliste"]
#[tokio::test]
async fn test_session_pseudonym() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let uuid = create_session(&app_state).await?;
    Ok(())
}

#[ignore = "Require mainzelliste"]
#[tokio::test]
async fn test_token_pseudonym() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let session_id = create_session(&app_state).await?;
    let token = create_token(&app_state, &session_id, 6).await?;
    Ok(())
}

#[tokio::test]
async fn test_create_patient() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let session_id = create_session(&app_state).await?;
    let token: CreateTokenResp = create_token(&app_state, &session_id, 1).await?;
    let psy = create_patient(&app_state, &token.id, "LOCAL_ID").await?;
    Ok(())
}

#[tokio::test]
async fn test_create_patients() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let patient_ids: Vec<String> = vec![
        "PATIENT-1".to_string(),
        "PATIENT-2".to_string(),
        "PATIENT-3".to_string(),
        "PATIENT-4".to_string(),
        "PATIENT-5".to_string(),
        "PATIENT-6".to_string(),
    ];
    let session_id = create_session(&app_state).await?;
    let token: CreateTokenResp = create_token(&app_state, &session_id, 6).await?;
    let psy = create_patients(&app_state, &token.id, patient_ids).await?;
    Ok(())
}
