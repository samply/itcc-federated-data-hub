use crate::s3_client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tokio::io::{AsyncRead, AsyncWriteExt};
use tracing::{debug, info};

pub async fn save_files_s3(
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
    upload_to_s3(bucket, filename, path).await?;
    Ok(())
}

pub async fn upload_to_s3(
    bucket: &str,
    filename: &str,
    path_buf: PathBuf,
) -> Result<(), anyhow::Error> {
    debug!("[Beam] Saving file to s3...");
    let client: &Client = s3_client().await;
    debug!("creating bytestream from path");
    let body = ByteStream::from_path(&path_buf).await?;
    client
        .put_object()
        .bucket(bucket)
        .key(filename)
        .content_type("text/plain; charset=utf-8")
        .body(body)
        .send()
        .await?;
    info!("s3 saved");
    Ok(())
}
pub async fn get_object(bucket: &str, filename: &str) -> anyhow::Result<()> {
    let client: &Client = s3_client().await;
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(filename)
        .send()
        .await?;
    debug!("s3 response: {:?}", resp);
    Ok(())
}

pub async fn show_buckets() -> anyhow::Result<()> {
    let client: &Client = s3_client().await;
    let res = client.list_buckets().send().await?;
    debug!("s3 response: {:?}", res);
    Ok(())
}
