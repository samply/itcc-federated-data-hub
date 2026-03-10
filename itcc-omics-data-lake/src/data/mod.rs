use std::collections::HashSet;
use tokio::io::AsyncWriteExt;
pub mod handler;

use crate::cbio_portal::data::{
    ClinicalPatientData, ClinicalPatientRow, ClinicalSampleData, ClinicalSampleRow, Diagnosis,
    PatientId, SampleId,
};
use crate::cbio_portal::{generate_cbio_portal_data_min, generate_cbio_portal_meta_min};
use crate::data::handler::{decompress_zstd_to_tempfile, maf_to_parquet};
use itcc_omics_lib::beam::MetaData;
use itcc_omics_lib::s3::{get_object, upload_to_s3_from_path};
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};
use tokio::io::AsyncRead;
use tracing::debug;

pub async fn save_files_s3(
    client_s3: &aws_sdk_s3::Client,
    bucket: &str,
    mut incoming: impl AsyncRead + Unpin,
    filename: &str,
) -> anyhow::Result<()> {
    let tmp = NamedTempFile::new()?;
    let path: PathBuf = tmp.path().to_path_buf();
    debug!("tmp path = {}", path.display());
    let mut f = tokio::fs::File::create(&path).await?;
    let size_u64 = tokio::io::copy(&mut incoming, &mut f).await?;
    f.flush().await?;
    debug!("wrote {size_u64} bytes to temp");
    upload_to_s3_from_path(client_s3, bucket, filename, path).await?;
    Ok(())
}

pub async fn process_and_generate_data(
    s3_client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    meta_data: MetaData,
) -> anyhow::Result<()> {
    // working dir
    let work = TempDir::new()?;
    let work_path: &Path = work.path();
    let downloaded = get_object(s3_client, bucket, key).await?;

    let maf_path: PathBuf = if key.ends_with(".zst") || key.ends_with(".zstd") {
        decompress_zstd_to_tempfile(&downloaded)?
    } else {
        downloaded
    };

    let parquet_path = work_path.join("mutations.parquet");
    let sample_ids: HashSet<SampleId> = maf_to_parquet(Path::new(&maf_path), &parquet_path)?;
    let parquet_key = format!("{}/{}.parquet", meta_data.partner_id, meta_data.maf_id);
    upload_to_s3_from_path(s3_client, bucket, &parquet_key, &parquet_path).await?;

    let (sample_rows, patient_rows) = build_minimal_cbio_rows(&sample_ids)?;

    generate_cbio_portal_data_min(
        s3_client,
        bucket,
        work_path,
        &patient_rows,
        &sample_rows,
        &meta_data,
    )
    .await?;
    generate_cbio_portal_meta_min(s3_client, bucket, work_path, &meta_data).await?;
    Ok(())
}
pub fn build_minimal_cbio_rows(
    sample_ids: &HashSet<SampleId>,
) -> anyhow::Result<(ClinicalSampleData, ClinicalPatientData)> {
    let mut sample_rows = Vec::new();
    let mut patient_ids: HashSet<PatientId> = HashSet::new();

    for sample_id in sample_ids {
        let patient_id = sample_id.to_patient_id()?;
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
