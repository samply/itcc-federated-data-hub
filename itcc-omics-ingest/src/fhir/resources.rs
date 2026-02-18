#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use crate::utils::error_type::ErrorType;
use serde::{Deserialize, Serialize};
// --------------------
// Bundle + Entry types
// --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub resourceType: String,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub bundle_type: Option<String>,
    pub total: Option<u32>,
    pub entry: Option<Vec<BundleEntry>>,
}

impl Bundle {
    pub fn resources(&self) -> impl Iterator<Item = &Resource> {
        self.entry
            .as_deref()
            .into_iter()
            .flat_map(|v| v.iter())
            .map(|e| &e.resource)
    }

    pub fn patient(&self) -> Option<&Patient> {
        self.resources().find_map(|r| match r {
            Resource::Patient(p) => Some(p),
            _ => None,
        })
    }

    pub fn patient_info(&self) -> Option<(String, String)> {
        self.entry.as_ref()?.iter().find_map(|entry| {
            if let Resource::Patient(patient) = &entry.resource {
                Some((patient.id.clone()?, entry.fullUrl.clone()?))
            } else {
                None
            }
        })
    }

    pub fn conditions(&self) -> Vec<&Condition> {
        self.resources()
            .filter_map(|r| match r {
                Resource::Condition(c) => Some(c),
                _ => None,
            })
            .collect()
    }

    pub fn all_condition_subject_references(&self) -> Vec<&str> {
        self.entry
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .filter_map(|entry| match &entry.resource {
                Resource::Condition(c) => c.subject.as_ref().and_then(|r| r.reference.as_deref()),
                _ => None,
            })
            .collect()
    }

    pub fn rewrite_all_references(&mut self, from: &str, to: &str) {
        let Some(entries) = self.entry.as_mut() else {
            return;
        };

        for entry in entries {
            match &mut entry.resource {
                Resource::Patient(_) => {}

                Resource::Condition(c) => {
                    if let Some(subj) = c.subject.as_mut() {
                        subj.rewrite(from, to);
                    }
                }

                Resource::Observation(o) => {
                    if let Some(subj) = o.subject.as_mut() {
                        subj.rewrite(from, to);
                    }
                }

                Resource::Specimen(s) => {
                    if let Some(subj) = s.subject.as_mut() {
                        subj.rewrite(from, to);
                    }
                }

                Resource::Unknown => {}
            }
        }
    }

    pub fn rename_patient_id_everywhere(
        &mut self,
        old_id: &str,
        new_id: &str,
    ) -> Result<(), ErrorType> {
        let from_ref = format!("Patient/{}", old_id);
        let to_ref = format!("Patient/{}", new_id);

        self.rewrite_all_references(&from_ref, &to_ref);

        if let Some(entries) = self.entry.as_mut() {
            for entry in entries {
                if let Resource::Patient(p) = &mut entry.resource {
                    if p.id.as_deref() == Some(old_id) {
                        p.id = Some(new_id.to_string());
                    } else {
                        Err(ErrorType::FhirCheckError)?
                    }
                    if let Some(full) = entry.fullUrl.as_mut() {
                        *full = full.replace(
                            &format!("/Patient/{}", old_id),
                            &format!("/Patient/{}", new_id),
                        );
                    } else {
                        Err(ErrorType::FhirCheckError)?
                    }
                }
            }
        }
        Ok(())
    }

    pub fn contains_patient_id(&self, patient_id: &str) -> bool {
        let needle_ref = format!("Patient/{}", patient_id);

        let Some(entries) = &self.entry else {
            return false;
        };

        for entry in entries {
            if let Some(full) = &entry.fullUrl {
                if full.contains(patient_id) {
                    return true;
                }
            }

            match &entry.resource {
                Resource::Patient(p) => {
                    // Check resource id
                    if p.id.as_deref() == Some(patient_id) {
                        return true;
                    }

                    if let Some(ids) = &p.identifier {
                        for id in ids {
                            if id.value.as_deref() == Some(patient_id) {
                                return true;
                            }
                        }
                    }
                }

                Resource::Condition(c) => {
                    if c.id.as_deref() == Some(patient_id) {
                        return true;
                    }

                    if let Some(subj) = &c.subject {
                        if subj.reference.as_deref() == Some(&needle_ref) {
                            return true;
                        }
                    }

                    if let Some(exts) = &c.extension {
                        for ext in exts {
                            if let Some(vr) = &ext.valueReference {
                                if vr.reference.as_deref() == Some(&needle_ref) {
                                    return true;
                                }
                            }
                        }
                    }
                }

                Resource::Observation(o) => {
                    if o.id.as_deref() == Some(patient_id) {
                        return true;
                    }

                    if let Some(subj) = &o.subject {
                        if subj.reference.as_deref() == Some(&needle_ref) {
                            return true;
                        }
                    }

                    if let Some(focus) = &o.focus {
                        for r in focus {
                            if r.reference.as_deref() == Some(&needle_ref) {
                                return true;
                            }
                        }
                    }
                }

                Resource::Specimen(s) => {
                    if s.id.as_deref() == Some(patient_id) {
                        return true;
                    }

                    if let Some(subj) = &s.subject {
                        if subj.reference.as_deref() == Some(&needle_ref) {
                            return true;
                        }
                    }

                    if let Some(ids) = &s.identifier {
                        for id in ids {
                            if id.value.as_deref() == Some(patient_id) {
                                return true;
                            }
                        }
                    }
                }

                Resource::Unknown => {}
            }
        }

        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleEntry {
    pub fullUrl: Option<String>,
    pub resource: Resource,
    pub search: Option<SearchInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInfo {
    pub mode: Option<String>,
}

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
    pub lastUpdated: Option<String>, // keep as String; parse to DateTime if you prefer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub system: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub reference: Option<String>, // e.g. "Patient/abcde1"
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
    pub gender: Option<String>, // "Male" / "Female" / ...
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationComponent {
    pub code: Option<CodeableConcept>,
    pub valueCodeableConcept: Option<CodeableConcept>,
    pub valueQuantity: Option<Quantity>,
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
}
