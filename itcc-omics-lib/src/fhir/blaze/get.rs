use crate::error_type::LibError;
use crate::patient_id::PatientId;
use beam_lib::reqwest::{StatusCode, Url};
use tracing::{debug, error, info};

pub async fn get_patient_fhir_by_id_json(
    client: &reqwest::Client,
    blaze_url: &Url,
    patient_id: &PatientId,
) -> Result<serde_json::Value, LibError> {
    let patient_url = blaze_url
        .join(format!("Patient?identifier={}&_revinclude=Condition:subject&_revinclude=Observation:subject&_revinclude=Specimen:subject", patient_id.as_str()).as_str())
        .expect("blaze url should be present");
    debug!("Patient: {}", patient_id);
    debug!("PatientUrl: {}", patient_url);
    let resp = client.get(patient_url).send().await.map_err(|e| {
        error!("Failed to get patient: {}", patient_id);
        error!("Error: {e}");
        LibError::BlazeError
    })?;
    let status = resp.status();
    if status == StatusCode::NOT_FOUND {
        return Err(LibError::FhirPatientNotFound);
    }
    let bundle = resp.json::<serde_json::Value>().await.map_err(|e| {
        error!("Failed to get patient: {}", patient_id);
        error!("Error: {e}");
        LibError::BlazeError
    })?;
    debug!("Patient result: {:#?}", bundle);
    Ok(bundle)
}
