use crate::fhir::bundle::Bundle;

use crate::MafTask;
use serde::{Deserialize, Serialize};

pub mod bundle;
pub mod resources;
pub mod blaze;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IngestTask {
    Fhir { bundle: Bundle },
    Maf(MafTask),
}
