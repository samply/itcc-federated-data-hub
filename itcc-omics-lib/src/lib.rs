use crate::patient_id::{PatientId, SampleId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[cfg(feature = "beam")]
pub mod beam;
#[cfg(feature = "s3")]
pub mod cbio_portal;
#[cfg(feature = "dwh")]
pub mod dwh;
pub mod error_type;
#[cfg(feature = "fhir")]
pub mod fhir;
#[cfg(feature = "ml")]
pub mod mainzelliste;
pub mod patient_id;
#[cfg(feature = "s3")]
pub mod s3;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaData {
    pub maf_id: String,
    pub origin_maf_id: String,
    pub partner_id: String,
    pub checked_fhir: bool,
    pub patient_sample_suffix: bool,
    pub patient_to_sample: HashMap<PatientId, HashSet<SampleId>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MafTask {
    pub meta: MetaData,
    pub suggested_name: Option<String>,
    pub bytes_b64: String,
}
