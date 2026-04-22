use crate::beam;
use crate::utils::config::AppState;
use crate::utils::error_type::ErrorType;
use itcc_omics_lib::error_type::LibError;
use itcc_omics_lib::fhir::blaze::get_patient_by_id;
use itcc_omics_lib::mainzelliste::handler::CreateTokenResp;
use itcc_omics_lib::mainzelliste::{encryption_ml, init_mainzelliste};
use itcc_omics_lib::patient_id::{filter_patient_id, insert_base, split_base, PatientId, SampleId};
use std::collections::{HashMap, HashSet};
use tracing::debug;

/// Builds a pseudonym mapping for a set of sample IDs by:
/// 1. Extracting patient IDs from the sample IDs
/// 2. Obtaining a Mainzelliste session token
/// 3. Encrypting patient IDs to cryptographic pseudonyms via Mainzelliste
/// 4. Fetching each patient's FHIR bundle from Blaze, rewriting the patient ID
///    to its pseudonym, and transmitting the bundle via Beam
/// 5. Constructing and returning a `HashMap<original_sample_id, pseudonymized_sample_id>`
///
/// # Errors
/// Returns [`ErrorType`] if Mainzelliste token creation, encryption, FHIR retrieval,
/// ID rewriting, Beam transmission, or pseudonym lookup fails.
pub async fn run_pseudonymisation(
    app_state: &AppState,
    sample_ids: &HashSet<SampleId>,
) -> Result<HashMap<PatientId, PatientId>, LibError> {
    // Mainzelliste
    let patients_id = filter_patient_id(sample_ids);
    let token: CreateTokenResp = init_mainzelliste(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
        patients_id.len(),
    )
    .await?;
    encryption_ml(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
        &token.id,
        &patients_id,
    )
    .await
}

pub async fn fhir_collector_sender(
    app_state: &AppState,
    local_crypto_ids: &HashMap<PatientId, PatientId>,
) -> Result<(), ErrorType> {
    for (patient_id, pseudo_id) in local_crypto_ids.iter() {
        debug!("Patient: {}", patient_id);
        debug!("Pseudo: {}", pseudo_id);
        let mut bundle =
            get_patient_by_id(&app_state.http, &app_state.services.blaze_url, patient_id).await?;
        bundle.rename_patient_id_everywhere(patient_id, pseudo_id)?;
        debug!("Bundle: {:#?}", bundle);
        beam::send_fhir_bundle(&app_state, bundle).await?;
    }
    Ok(())
}

pub async fn build_pseudo_map(
    sample_ids: &HashSet<SampleId>,
    local_crypto_ids: &HashMap<PatientId, PatientId>,
) -> Result<HashMap<SampleId, SampleId>, ErrorType> {
    let mut mapping_ids: HashMap<SampleId, SampleId> = HashMap::new();
    debug!("{:#?}", sample_ids);
    for sample in sample_ids {
        let base: PatientId = sample.to_patient_id();

        let crypto: PatientId = local_crypto_ids
            .get(&base)
            .ok_or(ErrorType::PseudoError)?
            .clone();

        let mapped = sample.to_pseudo_sample_id(crypto)?;

        mapping_ids.insert(sample.clone(), mapped);
    }
    Ok(mapping_ids)
}
