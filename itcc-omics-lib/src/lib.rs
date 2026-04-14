use serde::{Deserialize, Serialize};

#[cfg(feature = "beam")]
pub mod beam;
#[cfg(feature = "s3")]
pub mod cbio_portal;
pub mod error_type;
#[cfg(feature = "fhir")]
pub mod fhir;
#[cfg(feature = "ml")]
pub mod mainzelliste;
#[cfg(feature = "parquet")]
pub mod parquet;
pub mod patient_id;
#[cfg(feature = "s3")]
pub mod s3;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaData {
    pub maf_id: String,
    pub origin_maf_id: String,
    pub partner_id: String,
    pub checked_fhir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MafTask {
    pub meta: MetaData,
    pub suggested_name: Option<String>,
    pub bytes_b64: String,
}
