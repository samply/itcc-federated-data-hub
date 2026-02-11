use crate::omics_data::transfer::{filter_patient_id, insert_base};
use crate::pseudonym::handler::{
    create_patients, create_session, create_token, CreatePatientResp, CreateTokenResp,
};
use crate::utils::config::AppState;
use crate::utils::error_type::ErrorType;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::Hash;

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
    let local_crypto_ids: HashMap<String, String> = pseudonym_res
        .into_iter()
        .map(|r| extract_mapping(&r).unwrap())
        .collect();

    let mut mapping_ids: HashMap<String, String> = HashMap::new();

    for key in sample_ids {
        mapping_ids.insert(
            key.clone(),
            insert_base(&key, local_crypto_ids.get(&key).unwrap()).to_string(),
        );
    }
    Ok(mapping_ids)
}

fn extract_mapping(resp: &CreatePatientResp) -> Option<(String, String)> {
    let local = resp
        .iter()
        .find(|x| x.id_type == "localid")
        .map(|x| x.id_string.clone())?;

    let crypto = resp
        .iter()
        .find(|x| x.id_type == "cryptoid")
        .map(|x| x.id_string.clone())?;

    Some((local, crypto))
}
