use std::collections::HashSet;

pub fn filter_patient_id(ids: &HashSet<String>) -> HashSet<String> {
    ids.into_iter().map(|id| split_base(id)).collect()
}

pub fn split_base(sample: &str) -> String {
    sample
        .split_once("_")
        .map(|x| x.0)
        .unwrap_or(sample)
        .to_string()
}

pub fn insert_base(sample: &str, crypto_id: &str) -> String {
    sample
        .split_once("_")
        .map(|x| format!("{}_{}", crypto_id, x.1))
        .unwrap_or(crypto_id.to_string())
        .to_string()
}
