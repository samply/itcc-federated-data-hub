use tokio::io::AsyncWriteExt;
mod handler;

use crate::data::handler::{decompress_zstd_to_tempfile, maf_to_parquet};
use itcc_omics_lib::s3::{get_object, upload_to_s3};
use itcc_omics_lib::MetaData;
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
    upload_to_s3(client_s3, bucket, filename, path).await?;
    Ok(())
}

pub async fn process_maf_object_to_parquet(
    s3_client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    meta_data: MetaData,
) -> anyhow::Result<()> {
    // working dir
    let work = TempDir::new()?;
    let work_path = work.path();
    let downloaded = get_object(s3_client, bucket, key).await?;

    let maf_path: PathBuf = if key.ends_with(".zst") || key.ends_with(".zstd") {
        decompress_zstd_to_tempfile(&downloaded)?
    } else {
        downloaded
    };

    let parquet_path = work_path.join("mutations.parquet");
    maf_to_parquet(Path::new(&maf_path), &parquet_path)?;
    let parquet_key = format!("{}/{}.parquet", meta_data.partner_id, meta_data.maf_id);

    upload_to_s3(s3_client, bucket, &parquet_key, &parquet_path).await?;
    Ok(())
}
