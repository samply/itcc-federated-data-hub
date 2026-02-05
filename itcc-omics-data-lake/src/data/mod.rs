mod cbio;
mod handler;

use crate::data::handler::{decompress_zstd_to_tempfile, maf_to_parquet};
use crate::s3::{get_object, upload_to_s3};
use itcc_omics_lib::MetaData;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub async fn process_maf_object_to_parquet_and_cbio(
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

    // cBioPortal files
    let cbio_data = work_path.join("data_mutations_extended.txt");
    let cbio_meta = work_path.join("meta_mutations_extended.txt");

    //copy_maf_to_cbio(Path::new(&maf_path), &cbio_data)?;
    //write_cbio_meta(&cbio_meta, "data_mutations_extended.txt")?;

    let parquet_key = format!("{}.parquet", meta_data.maf_id);
    let cbio_data_key = format!("{}/data_mutations_extended.txt", meta_data.maf_id);
    let cbio_meta_key = format!("{}/meta_mutations_extended.txt", meta_data.maf_id);

    upload_to_s3(bucket, &parquet_key, &parquet_path).await?;
    //upload_to_s3(bucket, &cbio_data_key, &cbio_data).await?;
    //upload_to_s3(bucket, &cbio_meta_key, &cbio_meta).await?;

    Ok(())
}
