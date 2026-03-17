use crate::beam;
use crate::fhir::handler::get_patient_by_id;
use crate::omics_data::transfer::{filter_patient_id, insert_base, split_base};
use crate::pseudonym::handler::{
    create_patients, create_session, create_token, CreatePatientResp, CreateTokenResp,
};
use crate::utils::config::AppState;
use crate::utils::error_type::ErrorType;
use std::collections::{HashMap, HashSet};
use tracing::debug;

pub mod handler;

pub async fn build_pseudo_map(
    state: &AppState,
    sample_ids: HashSet<String>,
) -> Result<HashMap<String, String>, ErrorType> {
    let patients_id = filter_patient_id(&sample_ids);
    let session_id = create_session(&state).await?;
    let token: CreateTokenResp = create_token(&state, &session_id, patients_id.len()).await?;
    let pseudonym_res: Vec<CreatePatientResp> =
        create_patients(&state, &token.id, patients_id).await?;
    let local_crypto_ids: HashMap<String, String> = extract_mapping(pseudonym_res)?;

    debug!("Mapping: {:#?}", local_crypto_ids);
    // fhir handling
    for (patient_id, pseudo_id) in local_crypto_ids.iter() {
        debug!("Patient: {}", patient_id);
        debug!("Pseudo: {}", pseudo_id);
        let mut bundle = get_patient_by_id(&state, patient_id.as_str()).await?;
        bundle.rename_patient_id_everywhere(patient_id, pseudo_id)?;
        debug!("Bundle: {:#?}", bundle);
        beam::send_fhir_bundle(&state, bundle).await?;
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

fn extract_mapping(resp: Vec<CreatePatientResp>) -> Result<HashMap<String, String>, ErrorType> {
    resp.into_iter()
        .map(|r| {
            let local = r
                .iter()
                .find(|x| x.id_type == "localid")
                .map(|x| x.id_string.clone())
                .ok_or(ErrorType::PseudoError)?;

            let crypto = r
                .iter()
                .find(|x| x.id_type == "cryptoid")
                .map(|x| x.id_string.clone())
                .ok_or(ErrorType::PseudoError)?;

            Ok((local, crypto))
        })
        .collect::<Result<HashMap<String, String>, ErrorType>>()
}
