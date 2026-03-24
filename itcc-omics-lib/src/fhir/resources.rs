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
    pub fn id(&self) -> &str {
        match self {
            Resource::Patient(r) => r.id.as_str(),
            Resource::Condition(r) => r.id.as_str(),
            Resource::Observation(r) => r.id.as_str(),
            Resource::Specimen(r) => r.id.as_str(),
            Resource::Unknown => "unknown",
        }
    }
}

// --------------------
// Common building blocks
// --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    #[serde(rename = "versionId", skip_serializing_if = "Option::is_none")]
    pub versionId: Option<String>,

    #[serde(rename = "lastUpdated", skip_serializing_if = "Option::is_none")]
    pub lastUpdated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeableConcept {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coding: Option<Vec<Coding>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Age {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valueReference: Option<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valueAge: Option<Age>,
}

// --------------------
// Patient
// --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    pub id: String,
    pub identifier: Vec<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
}

// --------------------
// Condition
// --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onsetAge: Option<OnsetAge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<Vec<Extension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Reference>,
}

impl Condition {
    pub fn subject_reference(&self) -> Option<&str> {
        self.subject.as_ref()?.reference.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnsetAge {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
}

// --------------------
// Observation
// --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valueCodeableConcept: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus: Option<Vec<Reference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<Vec<ObservationComponent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationComponent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valueCodeableConcept: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valueQuantity: Option<Quantity>,
}

// --------------------
// Specimen
// --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specimen {
    pub id: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub specimen_type: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<Vec<Extension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<Vec<Identifier>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Reference>,
}
