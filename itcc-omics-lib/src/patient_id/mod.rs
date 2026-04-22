use crate::error_type::LibError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PatientId(String);

impl PatientId {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        Self(value)
    }
    pub fn as_str(&self) -> &str {
        &self.0.as_str()
    }
}

impl Display for PatientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SampleId(String);

impl SampleId {
    pub fn new(value: impl Into<String>) -> Result<Self, LibError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(LibError::SampleIdEmpty);
        }
        if !value.contains('_') {
            return Err(LibError::SampleIdInvalidFormat(value));
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0.as_str()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
    pub fn to_patient_id(&self) -> PatientId {
        PatientId::new(split_base(self.0.as_str()))
    }
    pub fn to_pseudo_sample_id(&self, crypto_id: PatientId) -> Result<SampleId, LibError> {
        SampleId::new(
            self.as_str()
                .split_once("_")
                .map(|x| format!("{}_{}", crypto_id, x.1))
                .unwrap_or(crypto_id.to_string())
                .to_string(),
        )
    }
}

impl Display for SampleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.as_str())
    }
}

pub fn filter_patient_id(sample_set: &HashSet<SampleId>) -> HashSet<PatientId> {
    sample_set
        .iter()
        .filter_map(|s| Some(s.to_patient_id()))
        .collect()
}

pub fn split_base(sample: &str) -> String {
    sample
        .split_once("_")
        .map(|x| x.0)
        .unwrap_or(sample)
        .to_string()
}

pub fn insert_base(sample: &SampleId, crypto_id: &PatientId) -> String {
    sample
        .as_str()
        .split_once("_")
        .map(|x| format!("{}_{}", crypto_id, x.1))
        .unwrap_or(crypto_id.to_string())
        .to_string()
}

pub fn patient_grouped_sample_id(
    sample_ids_pseudonym_sample_ids: &HashMap<SampleId, SampleId>,
    local_pseudonym_ids: &HashMap<PatientId, PatientId>,
) -> HashMap<PatientId, HashSet<SampleId>> {
    let mut patient_to_sample: HashMap<PatientId, HashSet<SampleId>> = HashMap::new();

    for (raw_sample, pseudo_sample) in sample_ids_pseudonym_sample_ids {
        let raw_patient = raw_sample.to_patient_id();
        if let Some(pseudo_patient) = local_pseudonym_ids.get(&raw_patient) {
            patient_to_sample
                .entry(pseudo_patient.clone())
                .or_default()
                .insert(pseudo_sample.clone());
        }
    }
    patient_to_sample
}
