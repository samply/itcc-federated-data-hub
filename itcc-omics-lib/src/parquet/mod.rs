mod handler;

use crate::cbio_portal::data::{
    ClinicalPatientData, ClinicalPatientRow, ClinicalSampleData, ClinicalSampleRow, Diagnosis,
    PatientId, SampleId,
};
use crate::cbio_portal::{
    build_minimal_cbio_rows, generate_cbio_portal_data_min, generate_cbio_portal_meta_min,
};
use crate::parquet::handler::maf_to_parquet;
use crate::s3::{decompress_zstd_to_tempfile, get_object, upload_to_s3_from_path};
use crate::MetaData;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};
use tokio::io::AsyncRead;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

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
    info!("downloaded maf");
    let maf_path: PathBuf = if key.ends_with(".zst") || key.ends_with(".zstd") {
        decompress_zstd_to_tempfile(&downloaded)?
    } else {
        downloaded
    };
    let parquet_path = work_path.join("mutation.parquet");
    let sample_ids: HashSet<SampleId> = maf_to_parquet(Path::new(&maf_path), &parquet_path)?;
    let parquet_key = format!(
        "{}/analytics/{}.parquet",
        meta_data.partner_id, meta_data.maf_id
    );
    upload_to_s3_from_path(s3_client, bucket, &parquet_key, &parquet_path).await?;
    info!("uploading parquet to s3://{bucket}/analytics/{parquet_key}");
    // generate_all_cbio_portal(
    //     s3_client,
    //     bucket,
    //     sample_ids,
    //     meta_data,
    //     work_path,
    // ).await?;
    Ok(())
}

pub async fn generate_all_cbio_portal(
    s3_client: &aws_sdk_s3::Client,
    bucket: &str,
    sample_ids: HashSet<SampleId>,
    meta_data: MetaData,
    work_path: &Path,
) -> anyhow::Result<()> {
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
