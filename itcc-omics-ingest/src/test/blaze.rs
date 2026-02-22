use crate::fhir::handler::{filter_patient_id_from_bundle, get_patient_by_id};
use crate::test::{test_app_state, test_config};
use crate::utils::error_type::ErrorType;
use reqwest::Client;
use tracing::debug;

// #[tokio::test]
async fn check_blaze() -> Result<(), reqwest::Error> {
    let cfg = test_config();
    let client = Client::new();
    let url = format!("{}health", cfg.blaze_url);
    debug!("URl: {}", url);

    let res = client.get(url).send().await?;

    debug!("status: {}", res.status());
    let body = res.text().await?;
    debug!("body: {}", body);

    Ok(())
}
// #[tokio::test]
async fn get_blaze_patient_by_id() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let id = "patient-001";
    let res = get_patient_by_id(&app_state, id).await?;
    //debug!("{:#?}", res);
    Ok(())
}

// #[tokio::test]
async fn check_blaze_patient_id() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let id = "patient-001";
    let res = get_patient_by_id(&app_state, id).await?;
    //debug!("{:#?}", res);
    filter_patient_id_from_bundle(res).await?;
    Ok(())
}
// #[tokio::test]
async fn test_blaze_pseudo() -> Result<(), ErrorType> {
    let patient_id = "patient-001";
    let pseudonym = "test-000";
    let app_state = test_app_state();
    let mut bundle = get_patient_by_id(&app_state, patient_id).await?;
    bundle.rename_patient_id_everywhere(patient_id, pseudonym);
    debug!("{:#?}", bundle);
    assert!(bundle.patient_info().unwrap().0 == "test-000");
    Ok(())
}

#[tokio::test]
async fn check_blaze_pseudo() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let mut bundle = get_patient_by_id(&app_state, "patient-001").await?;

    bundle.rename_patient_id_everywhere("patient-001", "test-000");

    assert!(
        !bundle.contains_patient_id("patient-001"),
        "SECURITY ERROR: Original patient ID still present in bundle!"
    );

    Ok(())
}
