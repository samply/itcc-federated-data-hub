use crate::utils::config::AppState;
use crate::utils::error_type::ErrorType;
use itcc_omics_lib::fhir::bundle::Bundle;
use itcc_omics_lib::fhir::resources::Resource;
use reqwest::StatusCode;
use tracing::debug;

pub async fn get_patient_by_id(state: &AppState, patient_id: &str) -> Result<Bundle, ErrorType> {
    let patient_url = state
        .services
        .blaze_url
        .join(&format!("fhir/Patient/{}/$everything", patient_id))
        .expect("blaze url should be present");

    let resp = state
        .http
        .get(patient_url)
        .send()
        .await
        .map_err(|_| ErrorType::BlazeError)?;
    let status = resp.status();

    if status.is_success() {
        let bundle = resp
            .json::<Bundle>()
            .await
            .map_err(|_| ErrorType::BlazeError)?;
        debug!("patient_details: {:?}", bundle);
        return Ok(bundle);
    }
    if status == StatusCode::NOT_FOUND {
        return Err(ErrorType::FhirPatientNotFound);
    } else {
        return Err(ErrorType::BlazeError);
    }
}

pub async fn pseudomize_patient_by_id(
    state: &AppState,
    patient_id: &str,
    pseudonym: &str,
) -> Result<Bundle, ErrorType> {
    let patient_url = state
        .services
        .blaze_url
        .join(&format!("fhir/Patient/{}/$everything", patient_id))
        .expect("blaze url should be present");

    let mut bundle: Bundle = state
        .http
        .get(patient_url)
        .send()
        .await
        .map_err(|_| ErrorType::BlazeError)?
        .error_for_status()
        .map_err(|_| ErrorType::BlazeError)?
        .json::<Bundle>()
        .await
        .map_err(|_| ErrorType::BlazeError)?;
    bundle.rename_patient_id_everywhere(patient_id, pseudonym);
    Ok(bundle)
}

pub async fn filter_patient_id_from_bundle(bundle: Bundle) -> Result<Bundle, ErrorType> {
    if let Some(entries) = &bundle.entry {
        for entry in entries {
            if let Resource::Condition(condition) = &entry.resource {
                if let Some(reference) = condition
                    .subject
                    .as_ref()
                    .and_then(|r| r.reference.as_ref())
                {
                    debug!("Subject reference: {}", reference);
                }
            }
        }
    }

    if let Some(p) = bundle.patient_info() {
        debug!("{:?}", p);
    }
    for r in bundle.all_condition_subject_references() {
        debug!("Condition subject.reference = {}", r);
    }
    Ok(bundle)
}
