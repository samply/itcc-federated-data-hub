pub mod client;

use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

pub async fn upload_to_s3(
    client_s3: &Client,
    bucket: &str,
    filename: &str,
    path: impl AsRef<Path>,
) -> Result<(), anyhow::Error> {
    debug!("[Beam] Saving file to s3...");
    debug!("creating bytestream from path");
    let body = ByteStream::from_path(&path).await?;
    client_s3
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
pub async fn get_object(
    client_s3: &Client,
    bucket: &str,
    filename: &str,
) -> anyhow::Result<PathBuf> {
    let resp = client_s3
        .get_object()
        .bucket(bucket)
        .key(filename)
        .send()
        .await?;
    debug!("s3 response: {:?}", resp);
    let mut body = resp.body.into_async_read();

    let tmp = NamedTempFile::new()?;
    let path: PathBuf = tmp.path().to_path_buf();
    let (_file, path) = tmp.keep()?;

    let mut out = tokio::fs::File::create(&path).await?;
    tokio::io::copy(&mut body, &mut out).await?;
    out.flush().await?;

    Ok(path)
}
