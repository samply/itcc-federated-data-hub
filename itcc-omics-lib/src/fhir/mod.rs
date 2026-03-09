use crate::fhir::bundle::Bundle;

use crate::beam::MafTask;
use serde::{Deserialize, Serialize};

pub mod blaze;
pub mod bundle;
pub mod resources;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IngestTask {
    Fhir { bundle: Bundle },
    Maf(MafTask),
}
