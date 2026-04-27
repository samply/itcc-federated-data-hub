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
