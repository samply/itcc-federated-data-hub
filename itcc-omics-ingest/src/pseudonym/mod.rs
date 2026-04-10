use crate::beam;
use crate::utils::config::AppState;
use crate::utils::error_type::ErrorType;
use itcc_omics_lib::fhir::blaze::get_patient_by_id;
use itcc_omics_lib::mainzelliste::handler::CreateTokenResp;
use itcc_omics_lib::mainzelliste::{encryption_ml, init_mainzelliste};
use itcc_omics_lib::patient_id::{filter_patient_id, insert_base, split_base};
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
pub async fn build_pseudo_map(
    app_state: &AppState,
    sample_ids: HashSet<String>,
) -> Result<HashMap<String, String>, ErrorType> {
    // Mainzelliste
    let patients_id = filter_patient_id(&sample_ids);
    let token: CreateTokenResp = init_mainzelliste(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
        patients_id.len(),
    )
    .await?;
    let local_crypto_ids = encryption_ml(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
        &token.id,
        patients_id,
    )
    .await?;
    // fhir handling
    for (patient_id, pseudo_id) in local_crypto_ids.iter() {
        debug!("Patient: {}", patient_id);
        debug!("Pseudo: {}", pseudo_id);
        let mut bundle = get_patient_by_id(
            &app_state.http,
            &app_state.services.blaze_url,
            patient_id.as_str(),
        )
        .await?;
        bundle.rename_patient_id_everywhere(patient_id, pseudo_id)?;
        debug!("Bundle: {:#?}", bundle);
        beam::send_fhir_bundle(&app_state, bundle).await?;
    }

    let mut mapping_ids: HashMap<String, String> = HashMap::new();

    debug!("{:#?}", sample_ids);
    for sample in sample_ids {
        let base = split_base(&sample);

        let crypto = local_crypto_ids.get(&base).ok_or(ErrorType::PseudoError)?;

        let mapped = insert_base(&sample, crypto);

        mapping_ids.insert(sample, mapped);
    }

    Ok(mapping_ids)
}
