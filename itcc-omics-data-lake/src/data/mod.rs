mod handler;

use crate::data::handler::{decompress_zstd_to_tempfile, maf_to_parquet};
use crate::s3::{get_object, upload_to_s3};
use itcc_omics_lib::MetaData;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub async fn process_maf_object_to_parquet(
    bucket: &str,
    key: &str,
    meta_data: MetaData,
) -> anyhow::Result<()> {
    // working dir
    let work = TempDir::new()?;
    let work_path = work.path();
    let downloaded = get_object(bucket, key).await?;

    let maf_path: PathBuf = if key.ends_with(".zst") || key.ends_with(".zstd") {
        decompress_zstd_to_tempfile(&downloaded)?
    } else {
        downloaded
    };

    let parquet_path = work_path.join("mutations.parquet");
    maf_to_parquet(Path::new(&maf_path), &parquet_path)?;
    let parquet_key = format!("{}/{}.parquet", meta_data.partner_id, meta_data.maf_id);

    upload_to_s3(bucket, &parquet_key, &parquet_path).await?;
    Ok(())
}
