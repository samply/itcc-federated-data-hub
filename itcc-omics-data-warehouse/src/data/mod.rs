use itcc_omics_lib::cbio_portal::data::{
    ClinicalPatientData, ClinicalPatientRow, ClinicalSampleData, ClinicalSampleRow, Diagnosis,
    PatientId, SampleId,
};
use itcc_omics_lib::cbio_portal::{generate_cbio_portal_data_min, generate_cbio_portal_meta_min};
use itcc_omics_lib::s3::{get_object, upload_to_s3_from_path};
use itcc_omics_lib::MetaData;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};
use tokio::io::AsyncRead;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};
pub mod handler;

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
