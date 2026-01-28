use crate::{s3_client, CONFIG};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tokio::io::{AsyncRead, AsyncWriteExt};
use tracing::info;

pub async fn upload_stream_to_s3(
    bucket: &str,
    key: &str,
    mut incoming: impl AsyncRead + Unpin,
) -> anyhow::Result<()> {
    let tmp = NamedTempFile::new()?;
    let path: PathBuf = tmp.path().to_path_buf();
    info!("tmp path = {}", path.display());

    let mut f = tokio::fs::File::create(&path).await?;
    info!("copying incoming -> temp");
    let size_u64 = tokio::io::copy(&mut incoming, &mut f).await?;
    f.flush().await?;
    info!("wrote {size_u64} bytes to temp");
    info!("[Beam] Saving file to s3...");
    let client: &Client = s3_client().await;
    info!("creating bytestream from path");
    let body = ByteStream::from_path(&path).await?;
    info!("bytestream created");
    info!("S3 access_key_id={}", CONFIG.s3_access_key_id);
    info!("S3 endpoint={}", CONFIG.s3_endpoint_url);
    info!("S3 bucket={}", CONFIG.s3_bucket);
    client.list_buckets().send().await?;
    info!("list_buckets ok");
    info!("put_object sending...");
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type("text/plain; charset=utf-8")
        .body(body)
        .send()
        .await?;
    info!("[Beam] s3 saved");
    Ok(())
}
