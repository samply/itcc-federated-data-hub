use crate::error_type::LibError;
use crate::fhir::resources::{Condition, Patient, Resource};
use crate::patient_id::PatientId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    #[serde(rename = "resourceType")]
    pub resourceType: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub bundle_type: Option<String>,

    pub total: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<Vec<BundleEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<Vec<BundleLink>>,
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
                Some((patient.id.clone(), entry.fullUrl.clone()?))
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
        old_id: &PatientId,
        new_id: &PatientId,
    ) -> Result<(), LibError> {
        if let Some(entries) = self.entry.as_mut() {
            for entry in entries {
                entry.search = None;
                match &mut entry.resource {
                    Resource::Patient(p) => {
                        p.identifier
                            .iter_mut()
                            .filter(|id| id.value == old_id.to_string())
                            .for_each(|id| id.value = new_id.to_string());

                        entry.request = Some(BundleRequest {
                            method: "PUT".to_string(),
                            url: format!("Patient/{}", p.id),
                        });
                    }

                    Resource::Condition(c) => {
                        entry.request = Some(BundleRequest {
                            method: "PUT".to_string(),
                            url: format!("Condition/{}", c.id),
                        });
                    }

                    Resource::Observation(o) => {
                        entry.request = Some(BundleRequest {
                            method: "PUT".to_string(),
                            url: format!("Observation/{}", o.id),
                        });
                    }

                    Resource::Specimen(s) => {
                        entry.request = Some(BundleRequest {
                            method: "PUT".to_string(),
                            url: format!("Specimen/{}", s.id),
                        });
                    }

                    Resource::Unknown => {}
                }
            }
        }
        // Importent to allow storing in DWH blaze
        self.id = None;
        self.bundle_type = Some("transaction".to_string());
        Ok(())
    }
    pub fn get_all_patient_identifiers(&self) -> HashSet<PatientId> {
        self.entry
            .iter()
            .flatten()
            .filter_map(|entry| match &entry.resource {
                Resource::Patient(p) => Some(p),
                _ => None,
            })
            .flat_map(|p| p.identifier.iter())
            .map(|id| PatientId::new(id.value.clone()))
            .collect()
    }

    // security check that fhir is pseudomised all fields
    pub fn contains_patient_id(&self, patient_id: &PatientId) -> bool {
        let needle_ref = format!("Patient/{}", patient_id);

        let Some(entries) = &self.entry else {
            return false;
        };

        for entry in entries {
            if let Some(full) = &entry.fullUrl {
                if full.contains(patient_id.as_str()) {
                    return true;
                }
            }

            match &entry.resource {
                Resource::Patient(p) => {
                    if let ids = &p.identifier {
                        for id in ids {
                            if id.value == patient_id.as_str() {
                                return true;
                            }
                        }
                    }
                }

                Resource::Condition(c) => {
                    if c.id == patient_id.as_str() {
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
                    if o.id == patient_id.as_str() {
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
                    if s.id == patient_id.as_str() {
                        return true;
                    }

                    if let Some(subj) = &s.subject {
                        if subj.reference.as_deref() == Some(&needle_ref) {
                            return true;
                        }
                    }

                    if let Some(ids) = &s.identifier {
                        for id in ids {
                            if id.value == patient_id.as_str() {
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

    // pageing over result bundle
    pub fn next_link(&self) -> Option<String> {
        self.link
            .iter()
            .flatten()
            .find(|l| l.relation == "next")
            .map(|l| l.url.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fullUrl: Option<String>,

    pub resource: Resource,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<SearchInfo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<BundleRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleRequest {
    pub method: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInfo {
    pub mode: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleLink {
    pub relation: String,
    pub url: String,
}
