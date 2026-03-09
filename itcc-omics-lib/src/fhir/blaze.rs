use crate::error_type::LibError;
use crate::fhir::bundle::Bundle;
use crate::fhir::resources::Resource;
use reqwest::{StatusCode, Url};
use tracing::debug;

pub async fn get_patient_by_id(
    client: &reqwest::Client,
    blaze_url: &Url,
    patient_id: &str,
) -> Result<Bundle, LibError> {
    let patient_url = blaze_url
        .join(&format!("Patient/{}/$everything", patient_id))
        .expect("blaze url should be present");
    debug!("Patient: {}", patient_id);
    debug!("PatientUrl: {}", patient_url);
    let resp = client
        .get(patient_url)
        .send()
        .await
        .map_err(|_| LibError::BlazeError)?;
    let status = resp.status();

    if status.is_success() {
        let bundle = resp
            .json::<Bundle>()
            .await
            .map_err(|_| LibError::BlazeError)?;
        debug!("patient_details: {:#?}", bundle);
        return Ok(bundle);
    }
    if status == StatusCode::NOT_FOUND {
        return Err(LibError::FhirPatientNotFound);
    } else {
        return Err(LibError::BlazeError);
    }
}

pub async fn pseudomize_patient_by_id(
    client: &reqwest::Client,
    blaze_url: &Url,
    patient_id: &str,
    pseudonym: &str,
) -> Result<Bundle, LibError> {
    let patient_url = blaze_url
        .join(&format!("Patient/{}/$everything", patient_id))
        .expect("blaze url should be present");
    debug!("pseudomize_patient_details: {:?}", patient_url);
    let mut bundle: Bundle = client
        .get(patient_url)
        .send()
        .await
        .map_err(|_| LibError::BlazeError)?
        .error_for_status()
        .map_err(|_| LibError::BlazeError)?
        .json::<Bundle>()
        .await
        .map_err(|_| LibError::BlazeError)?;
    debug!("patient_details: {:?}", bundle);
    bundle.rename_patient_id_everywhere(patient_id, pseudonym);
    Ok(bundle)
}

pub async fn filter_patient_id_from_bundle(bundle: Bundle) -> Result<Bundle, LibError> {
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
pub async fn post_patient_fhir_bundle(
    client: &reqwest::Client,
    blaze_url: &Url,
    bundle: &Bundle,
) -> Result<(), LibError> {
    debug!("PatientUrl: {:#?}", bundle);
    let resp = client
        .post(blaze_url.clone())
        .header("Content-Type", "application/fhir+json")
        .json(bundle)
        .send()
        .await
        .map_err(|_| LibError::BlazeError)?;
    let status = &resp.status();
    debug!("patient_details: {:#?}", status);

    if status.is_success() {
        let bundle = resp
            .json::<Bundle>()
            .await
            .map_err(|_| LibError::BlazeError)?;
        debug!("patient_details: {:#?}", bundle);
        return Ok(());
    }
    if status == &StatusCode::NOT_FOUND {
        return Err(LibError::FhirPatientNotFound);
    } else {
        return Err(LibError::BlazeError);
    }
    Ok(())
}
