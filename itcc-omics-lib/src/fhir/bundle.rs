use crate::error_type::LibError;
use crate::fhir::resources::{Condition, Patient, Resource};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    #[serde(rename = "resourceType")]
    pub resourceType: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub bundle_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<Vec<BundleEntry>>,

    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
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
    ) -> Result<(), LibError> {
        let from_ref = format!("Patient/{}", old_id);
        let to_ref = format!("Patient/{}", new_id);

        self.rewrite_all_references(&from_ref, &to_ref);

        if let Some(entries) = self.entry.as_mut() {
            for entry in entries {
                if let Resource::Patient(p) = &mut entry.resource {
                    if p.id.as_deref() == Some(old_id) {
                        p.id = Some(new_id.to_string());
                    } else {
                        Err(LibError::FhirCheckError)?;
                    }
                    if let Some(full) = entry.fullUrl.as_mut() {
                        *full = full.replace(
                            &format!("/Patient/{}", old_id),
                            &format!("/Patient/{}", new_id),
                        );
                    } else {
                        Err(LibError::FhirCheckError)?;
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
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInfo {
    pub mode: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
