use itcc_omics_lib::beam::MetaData;
use itcc_omics_lib::patient_id::split_base;
use itcc_omics_lib::s3::upload_to_s3_from_path;
use std::fmt::{self, Display};
use std::path::Path;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatientId(String);

impl PatientId {
    pub fn new(value: impl Into<String>) -> anyhow::Result<Self> {
        let value = value.into();
        anyhow::ensure!(!value.trim().is_empty(), "patient id must not be empty");
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_patient_id(&self) -> anyhow::Result<PatientId> {
        PatientId::new(split_base(self.as_str()))
    }
}

impl Display for PatientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SampleId(String);

impl SampleId {
    pub fn new(value: impl Into<String>) -> anyhow::Result<Self> {
        let value = value.into();
        anyhow::ensure!(!value.trim().is_empty(), "sample id must not be empty");
        Ok(Self(value))
    }
    pub fn to_patient_id(&self) -> anyhow::Result<PatientId> {
        PatientId::new(split_base(self.0.as_str()))
    }
}

impl Display for SampleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diagnosis {
    Aml,
    Dsrct,
    Epn,
    Ews,
    Hgg,
    Mb,
    Nbl,
    Nhl,
    Osteo,
    Other,
    Rhabdoid,
    Sts,
    Custom(String),
}

impl Display for Diagnosis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Diagnosis::Aml => f.write_str("AML"),
            Diagnosis::Dsrct => f.write_str("DSRCT"),
            Diagnosis::Epn => f.write_str("EPN"),
            Diagnosis::Ews => f.write_str("EWS"),
            Diagnosis::Hgg => f.write_str("HGG"),
            Diagnosis::Mb => f.write_str("MB"),
            Diagnosis::Nbl => f.write_str("NBL"),
            Diagnosis::Nhl => f.write_str("NHL"),
            Diagnosis::Osteo => f.write_str("Osteo"),
            Diagnosis::Other => f.write_str("Other"),
            Diagnosis::Rhabdoid => f.write_str("Rhabdoid"),
            Diagnosis::Sts => f.write_str("STS"),
            Diagnosis::Custom(value) => f.write_str(value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClinicalPatientRow {
    pub patient_id: PatientId,
    pub diagnosis: Diagnosis,
}

#[derive(Debug, Clone, Default)]
pub struct ClinicalPatientData {
    rows: Vec<ClinicalPatientRow>,
}

impl ClinicalPatientData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_rows(rows: Vec<ClinicalPatientRow>) -> Self {
        Self { rows }
    }

    pub fn push(&mut self, row: ClinicalPatientRow) {
        self.rows.push(row);
    }

    pub fn with_row(mut self, row: ClinicalPatientRow) -> Self {
        self.push(row);
        self
    }

    pub fn rows(&self) -> &[ClinicalPatientRow] {
        &self.rows
    }
}
impl CbioWritable for ClinicalPatientData {
    fn render(&self) -> String {
        let mut out = String::new();

        out.push_str("#Patient Identifier\tDiagnosis\n");
        out.push_str("#STRING\tSTRING\n");
        out.push_str("#1\t1\n");
        out.push_str("PATIENT_ID\tDIAGNOSIS\n");

        for row in &self.rows {
            out.push_str(&format!("{}\t{}\n", row.patient_id, row.diagnosis));
        }

        out
    }
}

#[derive(Debug, Clone)]
pub struct ClinicalSampleRow {
    pub sample_id: SampleId,
    pub patient_id: PatientId,
}

#[derive(Debug, Clone, Default)]
pub struct ClinicalSampleData {
    rows: Vec<ClinicalSampleRow>,
}

impl ClinicalSampleData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_rows(rows: Vec<ClinicalSampleRow>) -> Self {
        Self { rows }
    }

    pub fn push(&mut self, row: ClinicalSampleRow) {
        self.rows.push(row);
    }

    pub fn with_row(mut self, row: ClinicalSampleRow) -> Self {
        self.push(row);
        self
    }

    pub fn rows(&self) -> &[ClinicalSampleRow] {
        &self.rows
    }
}
impl CbioWritable for ClinicalSampleData {
    fn render(&self) -> String {
        let mut out = String::new();

        out.push_str("#Sample Identifier\tPatient Identifier\n");
        out.push_str("#STRING\tSTRING\n");
        out.push_str("#1\t1\n");
        out.push_str("SAMPLE_ID\tPATIENT_ID\n");

        for row in &self.rows {
            out.push_str(&format!("{}\t{}\n", row.sample_id, row.patient_id));
        }

        out
    }
}
