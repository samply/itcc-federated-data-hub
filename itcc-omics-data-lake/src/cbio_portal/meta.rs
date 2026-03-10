use std::fmt::{self, Display};
use std::path::Path;
use itcc_omics_lib::beam::MetaData;
use itcc_omics_lib::s3::upload_to_s3_from_path;
// --------------------
// shared
// --------------------

pub trait CbioWritable {
    fn render(&self) -> String;

    fn write_to(&self, path: &Path) -> anyhow::Result<()> {
        std::fs::write(path, self.render())?;
        Ok(())
    }

    async fn write_to_s3(
        &self,
        local_path: &Path,
        s3_client: &aws_sdk_s3::Client,
        bucket: &str,
        s3_key: &str,
    ) -> anyhow::Result<()> {
        self.write_to(local_path)?;
        upload_to_s3_from_path(s3_client, bucket, s3_key, local_path).await
    }
}

// --------------------
// study meta
// --------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeOfCancer {
    Mixed,
}

impl Display for TypeOfCancer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeOfCancer::Mixed => f.write_str("mixed"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetaStudy {
    pub cancer_study_identifier: String,
    pub type_of_cancer: TypeOfCancer,
    pub name: String,
    pub description: String,
    pub reference_genome: Option<String>,
    pub add_global_case_list: bool,
}

impl Default for MetaStudy {
    fn default() -> Self {
        Self {
            cancer_study_identifier: "itcc".to_string(),
            type_of_cancer: TypeOfCancer::Mixed,
            name: String::new(),
            description: String::new(),
            reference_genome: None,
            add_global_case_list: true,
        }
    }
}
impl CbioWritable for MetaStudy {
    fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "cancer_study_identifier: {}\n",
            self.cancer_study_identifier
        ));
        out.push_str(&format!("type_of_cancer: {}\n", self.type_of_cancer));
        out.push_str(&format!("name: {}\n", self.name));
        out.push_str(&format!("description: {}\n", self.description));

        if let Some(reference_genome) = &self.reference_genome {
            out.push_str(&format!("reference_genome: {}\n", reference_genome));
        }

        out.push_str(&format!(
            "add_global_case_list: {}\n",
            if self.add_global_case_list {
                "true"
            } else {
                "false"
            }
        ));

        out
    }
}

// --------------------
// clinical meta
// --------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClinicalDatatype {
    PatientAttributes,
    SampleAttributes,
}

impl Display for ClinicalDatatype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClinicalDatatype::PatientAttributes => f.write_str("PATIENT_ATTRIBUTES"),
            ClinicalDatatype::SampleAttributes => f.write_str("SAMPLE_ATTRIBUTES"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MetaClinical {
    Patient {
        study_id: String,
        data_filename: String,
    },
    Sample {
        study_id: String,
        data_filename: String,
    },
}

impl MetaClinical {
    pub fn patient(study_id: impl Into<String>) -> Self {
        Self::Patient {
            study_id: study_id.into(),
            data_filename: "data_clinical_patient.txt".into(),
        }
    }

    pub fn sample(study_id: impl Into<String>) -> Self {
        Self::Sample {
            study_id: study_id.into(),
            data_filename: "data_clinical_sample.txt".into(),
        }
    }
}

impl CbioWritable for MetaClinical {
    fn render(&self) -> String {
        match self {
            MetaClinical::Patient {
                study_id,
                data_filename,
            } => format!(
                concat!(
                    "cancer_study_identifier: {}\n",
                    "genetic_alteration_type: CLINICAL\n",
                    "datatype: {}\n",
                    "data_filename: {}\n",
                ),
                study_id,
                ClinicalDatatype::PatientAttributes,
                data_filename,
            ),
            MetaClinical::Sample {
                study_id,
                data_filename,
            } => format!(
                concat!(
                    "cancer_study_identifier: {}\n",
                    "genetic_alteration_type: CLINICAL\n",
                    "datatype: {}\n",
                    "data_filename: {}\n",
                ),
                study_id,
                ClinicalDatatype::SampleAttributes,
                data_filename,
            ),
        }
    }
}

// --------------------
// mutation meta
// --------------------

#[derive(Debug, Clone)]
pub struct MetaMutation {
    pub study_id: String,
    pub data_filename: String,
    pub stable_id: String,
    pub profile_name: String,
    pub profile_description: String,
    pub show_profile_in_analysis_tab: bool,
}

impl Default for MetaMutation {
    fn default() -> Self {
        Self {
            study_id: "itcc".to_string(),
            data_filename: "data_mutations.maf".to_string(),
            stable_id: "mutations".to_string(),
            profile_name: "Mutations".to_string(),
            profile_description: "WGS mutations".to_string(),
            show_profile_in_analysis_tab: true,
        }
    }
}

impl CbioWritable for MetaMutation {
    fn render(&self) -> String {
        format!(
            concat!(
                "cancer_study_identifier: {}\n",
                "genetic_alteration_type: MUTATION_EXTENDED\n",
                "datatype: MAF\n",
                "data_filename: {}\n",
                "stable_id: {}\n",
                "profile_name: {}\n",
                "profile_description: {}\n",
                "show_profile_in_analysis_tab: {}\n",
                "swissprot_identifier: name\n",
            ),
            self.study_id,
            self.data_filename,
            self.stable_id,
            self.profile_name,
            self.profile_description,
            if self.show_profile_in_analysis_tab {
                "true"
            } else {
                "false"
            },
        )
    }
}
