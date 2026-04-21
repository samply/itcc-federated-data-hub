use crate::cbio_portal::data::{ClinicalPatientRow, ClinicalSampleRow, Diagnosis};
use crate::patient_id::{PatientId, SampleId};
use crate::MetaData;
use data::CbioWritable as dataCbioWritable;
use data::{ClinicalPatientData, ClinicalSampleData};
use meta::CbioWritable as metaCbioWritable;
use meta::{MetaClinical, MetaMutation, MetaStudy};
use std::collections::HashSet;
use std::path::Path;

pub mod data;
pub mod meta;

pub fn build_minimal_cbio_rows(
    sample_ids: &HashSet<SampleId>,
) -> anyhow::Result<(ClinicalSampleData, ClinicalPatientData)> {
    let mut sample_rows = Vec::new();
    let mut patient_ids: HashSet<PatientId> = HashSet::new();

    for sample_id in sample_ids {
        let patient_id = sample_id.to_patient_id();
        patient_ids.insert(patient_id.clone());

        sample_rows.push(ClinicalSampleRow {
            sample_id: sample_id.clone(),
            patient_id,
        });
    }

    let mut patient_rows = Vec::new();

    for patient_id in patient_ids {
        patient_rows.push(ClinicalPatientRow {
            patient_id,
            diagnosis: Diagnosis::Custom("Other".to_string()),
        });
    }

    Ok((
        ClinicalSampleData::from_rows(sample_rows),
        ClinicalPatientData::from_rows(patient_rows),
    ))
}

pub async fn generate_cbio_portal_meta_min(
    s3_client: &aws_sdk_s3::Client,
    bucket: &str,
    path: &Path,
    meta_data: &MetaData,
) -> anyhow::Result<()> {
    let cbio_portal_base = format!("{}/{}/", meta_data.partner_id, meta_data.maf_id);
    MetaStudy {
        name: "INFORM Oncoanalyzer data".to_string(),
        description: "INFORM OA DATASET".to_string(),
        ..Default::default()
    }
    .write_to_s3(
        &path.join("meta_study.txt"),
        s3_client,
        bucket,
        &format!("{cbio_portal_base}meta_study.txt"),
    )
    .await?;

    MetaClinical::patient("itcc")
        .write_to_s3(
            &path.join("meta_clinical_patient.txt"),
            s3_client,
            bucket,
            &format!("{cbio_portal_base}meta_clinical_patient.txt"),
        )
        .await?;

    MetaClinical::sample("itcc")
        .write_to_s3(
            &path.join("meta_clinical_sample.txt"),
            s3_client,
            bucket,
            &format!("{cbio_portal_base}meta_clinical_sample.txt"),
        )
        .await?;

    MetaMutation {
        data_filename: format!("{}.maf", meta_data.maf_id),
        stable_id: "mutations".to_string(),
        profile_name: "Mutations".to_string(),
        profile_description: "WGS mutations".to_string(),
        show_profile_in_analysis_tab: true,
        ..Default::default()
    }
    .write_to_s3(
        &path.join("meta_mutation.txt"),
        s3_client,
        bucket,
        &format!("{cbio_portal_base}meta_mutation.txt"),
    )
    .await?;

    Ok(())
}

pub async fn generate_cbio_portal_data_min(
    s3_client: &aws_sdk_s3::Client,
    bucket: &str,
    path: &Path,
    clinical_patient_data: &ClinicalPatientData,
    clinical_sample_data: &ClinicalSampleData,
    meta_data: &MetaData,
) -> anyhow::Result<()> {
    let cbio_portal_base = format!("{}/{}/", meta_data.partner_id, meta_data.maf_id);

    clinical_patient_data
        .write_to_s3(
            &path.join("data_clinical_patient.txt"),
            s3_client,
            bucket,
            &format!("{cbio_portal_base}data_clinical_patient.txt"),
        )
        .await?;

    clinical_sample_data
        .write_to_s3(
            &path.join("data_clinical_sample.txt"),
            s3_client,
            bucket,
            &format!("{cbio_portal_base}data_clinical_sample.txt"),
        )
        .await?;

    Ok(())
}
