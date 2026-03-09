#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use crate::error_type::LibError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
// --------------------
// Resource enum (mixed)
// --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "resourceType")]
pub enum Resource {
    Patient(Patient),
    Condition(Condition),
    Observation(Observation),
    Specimen(Specimen),
    #[serde(other)]
    Unknown,
}

impl Resource {
    pub fn id(&self) -> Option<&str> {
        match self {
            Resource::Patient(r) => r.id.as_deref(),
            Resource::Condition(r) => r.id.as_deref(),
            Resource::Observation(r) => r.id.as_deref(),
            Resource::Specimen(r) => r.id.as_deref(),
            Resource::Unknown => None,
        }
    }
}

// --------------------
// Common building blocks
// --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub versionId: Option<String>,
    pub lastUpdated: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub system: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub reference: Option<String>,
}

impl Reference {
    pub fn rewrite(&mut self, from: &str, to: &str) {
        if self.reference.as_deref() == Some(from) {
            self.reference = Some(to.to_string());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coding {
    pub system: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeableConcept {
    pub coding: Option<Vec<Coding>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Age {
    pub value: Option<f64>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantity {
    pub value: Option<f64>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub url: String,
    pub valueReference: Option<Reference>,
    pub valueAge: Option<Age>,
}

// --------------------
// Patient
// --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    pub meta: Option<Meta>,
    pub id: Option<String>,
    pub identifier: Option<Vec<Identifier>>,
    pub gender: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

// --------------------
// Condition
// --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub meta: Option<Meta>,
    pub id: Option<String>,

    pub onsetAge: Option<OnsetAge>,
    pub extension: Option<Vec<Extension>>,
    pub code: Option<CodeableConcept>,
    pub subject: Option<Reference>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl Condition {
    pub fn subject_reference(&self) -> Option<&str> {
        self.subject.as_ref()?.reference.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnsetAge {
    pub value: Option<f64>,
}

// --------------------
// Observation
// --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub meta: Option<Meta>,
    pub id: Option<String>,

    pub code: Option<CodeableConcept>,
    pub valueCodeableConcept: Option<CodeableConcept>,
    pub subject: Option<Reference>,

    pub focus: Option<Vec<Reference>>,
    pub component: Option<Vec<ObservationComponent>>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationComponent {
    pub code: Option<CodeableConcept>,
    pub valueCodeableConcept: Option<CodeableConcept>,
    pub valueQuantity: Option<Quantity>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

// --------------------
// Specimen
// --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specimen {
    pub meta: Option<Meta>,
    pub id: Option<String>,

    #[serde(rename = "type")]
    pub specimen_type: Option<CodeableConcept>,

    pub extension: Option<Vec<Extension>>,
    pub identifier: Option<Vec<Identifier>>,
    pub subject: Option<Reference>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
