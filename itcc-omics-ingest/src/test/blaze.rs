use crate::test::{test_app_state, test_config};
use crate::utils::error_type::ErrorType;
use axum::http::StatusCode;
use beam_lib::reqwest::Url;
use itcc_omics_lib::error_type::LibError;
use itcc_omics_lib::fhir::blaze::{
    filter_patient_id_from_bundle, get_all_patient_count, get_all_patient_identifiers,
    get_patient_by_id, post_patient_fhir_bundle,
};
use itcc_omics_lib::fhir::bundle::Bundle;
use reqwest::Client;
use std::collections::HashMap;
use tracing::{debug, error};

#[ignore = "Require blaze"]
#[tokio::test]
async fn check_blaze() -> Result<(), reqwest::Error> {
    let cfg = test_config();
    let client = Client::new();
    let url = format!("http://localhost:8008/health");
    debug!("URl: {}", url);

    let res = client.get(url).send().await?;

    debug!("status: {}", res.status());
    let body = res.text().await?;
    debug!("body: {}", body);

    Ok(())
}

#[ignore = "Require blaze"]
#[tokio::test]
async fn get_blaze_patient_by_id() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let id = "patient-001";
    let res = get_patient_by_id(&app_state.http, &app_state.services.blaze_url, id).await?;
    debug!("{:#?}", res);
    Ok(())
}

#[ignore = "Require blaze"]
#[tokio::test]
async fn check_blaze_patient_id() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let id = "P0KRKM80V";
    let res = get_patient_by_id(&app_state.http, &app_state.services.blaze_url, id).await?;
    //debug!("{:#?}", res);
    filter_patient_id_from_bundle(res).await?;
    Ok(())
}

#[ignore = "Require blaze"]
#[tokio::test]
async fn test_blaze_pseudo() -> Result<(), ErrorType> {
    let patient_id = "P0KRKM80V";
    let pseudonym = "test-000";
    let app_state = test_app_state();
    let mut bundle =
        get_patient_by_id(&app_state.http, &app_state.services.blaze_url, patient_id).await?;
    bundle.rename_patient_id_everywhere(patient_id, pseudonym);
    debug!("{:#?}", bundle);
    assert!(bundle.patient_info().unwrap().0 == "test-000");
    Ok(())
}

#[ignore = "Require blaze"]
#[tokio::test]
async fn check_blaze_pseudo() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let mut bundle =
        get_patient_by_id(&app_state.http, &app_state.services.blaze_url, "P0KRKM80V").await?;

    bundle.rename_patient_id_everywhere("P0KRKM80V", "test-000");

    assert!(
        !bundle.contains_patient_id("patient-001"),
        "SECURITY ERROR: Original patient ID still present in bundle!"
    );

    Ok(())
}

#[ignore = "Require blaze"]
#[tokio::test]
async fn test_post_patient_fhir_bundle() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let bundle = Bundle {
        resourceType: "Bundle".to_string(),
        id: None,
        bundle_type: Some("transaction".to_string()),
        total: 0,
        entry: Some(vec![]),
    };
    let res =
        post_patient_fhir_bundle(&app_state.http, &app_state.services.blaze_url, &bundle).await?;
    debug!("{:#?}", res);
    Ok(())
}

pub async fn get_patient_by_id_debug(
    client: &reqwest::Client,
    blaze_url: &Url,
    patient_id: &str,
) -> Result<Bundle, LibError> {
    let patient_url = blaze_url
        .join(
            format!(
                "Patient?identifier={patient_id}\
            &_revinclude=Condition:subject\
            &_revinclude=Observation:subject\
            &_revinclude=Specimen:subject"
            )
            .as_str(),
        )
        .expect("blaze url should be present");

    debug!("Patient: {}", patient_id);
    debug!("PatientUrl: {}", patient_url);

    let resp = client.get(patient_url).send().await.map_err(|e| {
        error!("Failed to connect: {}", e);
        LibError::BlazeError
    })?;

    let status = resp.status();

    // Read body ONCE
    let body = resp.text().await.map_err(|_| LibError::BlazeError)?;

    if status == StatusCode::NOT_FOUND {
        return Err(LibError::FhirPatientNotFound);
    }

    if !status.is_success() {
        error!("Blaze error {}: {}", status, body);
        return Err(LibError::BlazeError);
    }

    // Parse as raw Value first for debugging
    let raw: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
        error!("Raw parse failed: {}", e);
        error!("Body snippet: {}", &body[..body.len().min(500)]);
        LibError::BlazeError
    })?;

    debug!(
        "Raw response: {}",
        serde_json::to_string_pretty(&raw).unwrap()
    );

    // Parse from Value — NOT from resp again!
    serde_json::from_value::<Bundle>(raw).map_err(|e| {
        error!("Bundle parse failed for patient {}: {}", patient_id, e);
        LibError::BlazeError
    })
}

#[ignore = "Require blaze"]
#[tokio::test]
async fn get_all_patients_count() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    get_all_patient_count(&app_state.http, &app_state.services.blaze_url).await?;
    Ok(())
}

#[ignore = "Require blaze"]
#[tokio::test]
async fn get_all_patient_identifiers_test() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    let count = get_all_patient_count(&app_state.http, &app_state.services.blaze_url).await?;
    let res =
        get_all_patient_identifiers(&app_state.http, &app_state.services.blaze_url, count).await?;
    debug!("{:#?}", res);
    Ok(())
}
