use crate::cbio_portal::data::CbioWritable as dataCbioWritable;
use crate::cbio_portal::meta::CbioWritable as metaCbioWritable;
use std::path::Path;
use itcc_omics_lib::beam::MetaData;
use crate::cbio_portal::data::{ClinicalPatientData, ClinicalSampleData};
use crate::cbio_portal::meta::{MetaClinical, MetaMutation, MetaStudy};

pub mod data;
pub mod meta;

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
        ).await?;

    MetaClinical::patient("itcc")
        .write_to_s3(
            &path.join("meta_clinical_patient.txt"),
            s3_client,
            bucket,
            &format!("{cbio_portal_base}meta_clinical_patient.txt"),
        ).await?;

    MetaClinical::sample("itcc")
        .write_to_s3(
            &path.join("meta_clinical_sample.txt"),
            s3_client,
            bucket,
        &format!("{cbio_portal_base}meta_clinical_sample.txt"),
        ).await?;

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
        ).await?;

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