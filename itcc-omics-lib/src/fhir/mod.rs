use crate::fhir::bundle::Bundle;
use serde::{Deserialize, Serialize};

pub mod bundle;
pub mod resources;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirBundleTask {
    pub bundle: Bundle,
}
