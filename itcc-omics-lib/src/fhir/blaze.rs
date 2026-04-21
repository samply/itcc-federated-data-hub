use crate::error_type::LibError;
use crate::fhir::bundle::Bundle;
use crate::fhir::resources::Resource;
use crate::patient_id::PatientId;
use reqwest::{StatusCode, Url};
use std::collections::HashSet;
use tracing::{debug, error, info};

pub async fn get_patient_by_id(
    client: &reqwest::Client,
    blaze_url: &Url,
    patient_id: &PatientId,
) -> Result<Bundle, LibError> {
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

    let bundle = resp.json::<Bundle>().await.map_err(|e| {
        error!("Failed to get patient: {}", patient_id);
        error!("Error: {e}");
        LibError::BlazeError
    })?;
    match bundle.patient() {
        Some(patient) => {
            info!("Patient: {:#?}", patient);
            if !bundle.contains_patient_id(patient_id) {
                return Err(LibError::FhirCheckError);
            }
            Ok(bundle)
        }
        None => Err(LibError::FhirPatientNotFound),
    }
}

pub async fn pseudomize_patient_by_id_transport(
    client: &reqwest::Client,
    blaze_url: &Url,
    patient_id: &PatientId,
    pseudonym: &PatientId,
) -> Result<Bundle, LibError> {
    let patient_url = blaze_url
        .join(&format!(
            "Patient?_id={}&_revinclude=Condition:subject&_revinclude=Observation:subject&_revinclude=Specimen:subject",
            patient_id
        ).as_str())
        .expect("blaze url should be present");
    let resp = client.get(patient_url).send().await.map_err(|e| {
        error!("Failed to get patient: {}", patient_id);
        error!("Error: {e}");
        LibError::BlazeError
    })?;
    let status = resp.status();
    if status == StatusCode::NOT_FOUND {
        return Err(LibError::FhirPatientNotFound);
    }

    let mut bundle = resp.json::<Bundle>().await.map_err(|e| {
        error!("Failed to get patient: {}", patient_id);
        error!("Error: {e}");
        LibError::BlazeError
    })?;
    match bundle.patient() {
        Some(patient) => {
            info!("Patient: {:#?}", patient);
            if !bundle.contains_patient_id(patient_id) {
                return Err(LibError::FhirCheckError);
            }
            bundle.rename_patient_id_everywhere(patient_id, pseudonym)?;
            Ok(bundle)
        }
        None => Err(LibError::FhirPatientNotFound),
    }
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
    let resp = client
        .post(blaze_url.clone())
        .header("Content-Type", "application/fhir+json")
        .json(bundle)
        .send()
        .await
        .map_err(|_| LibError::BlazeError)?;
    let status = &resp.status();

    if status.is_success() {
        let res = resp
            .json::<Bundle>()
            .await
            .map_err(|_| LibError::BlazeError)?;
        debug!("Post to blaze {:?}", res);
        return Ok(());
    }
    let body = resp
        .text()
        .await
        .unwrap_or_else(|_| "could not read body".to_string());
    if status == &StatusCode::NOT_FOUND {
        error!("Post to blaze {:?}", body);
        Err(LibError::FhirPatientNotFound)
    } else {
        error!("Post to blaze {:?}", body);
        Err(LibError::BlazeError)
    }
}

pub async fn get_all_patient_count(
    client: &reqwest::Client,
    blaze_url: &Url,
) -> Result<i64, LibError> {
    let patient_url = blaze_url
        .join("Patient?identifier:missing=false&_summary=count&_total=accurate")
        .expect("blaze url should be present");

    let resp: serde_json::Value = client
        .get(patient_url)
        .send()
        .await
        .map_err(|e| {
            error!("Error: {e}");
            LibError::BlazeError
        })?
        .error_for_status()
        .map_err(|_| LibError::BlazeError)?
        .json()
        .await
        .map_err(|_| LibError::BlazeError)?;
    debug!("resp: {:#?}", resp);
    let count: i64 = resp
        .get("total")
        .ok_or_else(|| LibError::BlazeError)?
        .as_i64()
        .ok_or_else(|| LibError::BlazeError)?;
    debug!("Count: {:?}", count);
    if count >= 100000 {
        // TODO paging offer 10000 patients
        return Err(LibError::BlazeResultError);
    } else {
        return Ok(count);
    }
}

pub async fn get_all_patient_identifiers(
    client: &reqwest::Client,
    blaze_url: &Url,
    counter: i64,
) -> Result<HashSet<PatientId>, LibError> {
    let mut identifiers = HashSet::new();
    let mut page = 0;
    let mut next_url: Option<String> = Some(
        blaze_url
            .join(
                format!(
                    "Patient?identifier:missing=false&_elements=identifier&_count={}",
                    counter
                )
                .as_str(),
            )
            .expect("blaze url should be present")
            .to_string(),
    );

    while let Some(url) = next_url {
        page += 1;
        debug!("Fetching patient identifiers page {page}: {url}");

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| LibError::BlazeConnectionError {
                url: url.clone(),
                message: e.to_string(),
            })?
            .error_for_status()
            .map_err(|e| LibError::BlazeConnectionError {
                url: url.clone(),
                message: e.to_string(),
            })?;

        let bundle: Bundle = resp
            .json::<Bundle>()
            .await
            .map_err(|e| LibError::BlazeParseError(e.to_string()))?;

        let page_identifiers = bundle.get_all_patient_identifiers();
        debug!(
            "Page {page}: got {} identifiers: {page_identifiers:?}",
            page_identifiers.len()
        );
        identifiers.extend(page_identifiers);

        next_url = bundle.next_link();
        debug!("Page {page}: next_url = {next_url:?}");
    }

    info!(
        "Fetched {} patient identifiers in {page} page(s)",
        identifiers.len()
    );
    Ok(identifiers)
}

pub async fn get_all_patient_identifiers_dbg(
    client: &reqwest::Client,
    blaze_url: &Url,
    count: i64,
) -> Result<HashSet<PatientId>, LibError> {
    let patient_url = blaze_url
        .join(
            format!("Patient?identifier:missing=false&_elements=identifier&_count={count}")
                .as_str(),
        )
        .expect("blaze url should be present");

    let res = client
        .get(patient_url)
        .send()
        .await
        .map_err(|_| LibError::BlazeError)?
        .error_for_status()
        .map_err(|_| LibError::BlazeError)?;

    debug!("resp: {:#?}", res);
    let bundle: Bundle = res
        .json::<Bundle>()
        .await
        .map_err(|_| LibError::BlazeError)?;
    Ok(bundle.get_all_patient_identifiers())
}
