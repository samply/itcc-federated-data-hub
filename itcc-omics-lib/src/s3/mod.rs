pub mod client;

use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

fn content_type_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("txt") | Some("tsv") | Some("maf") => "text/plain; charset=utf-8",
        Some("json") => "application/json",
        Some("parquet") => "application/octet-stream",
        Some("zst") | Some("zstd") => "application/zstd",
        _ => "application/octet-stream",
    }
}

async fn upload_body_to_s3(
    client_s3: &Client,
    bucket: &str,
    s3_key: &str,
    body: ByteStream,
    content_type: &'static str,
) -> anyhow::Result<()> {
    debug!("uploading to s3 key={s3_key} content_type={content_type}");

    client_s3
        .put_object()
        .bucket(bucket)
        .key(s3_key)
        .content_type(content_type)
        .body(body)
        .send()
        .await?;

    info!("s3 {s3_key} saved");
    Ok(())
}

pub async fn upload_to_s3_from_path(
    client_s3: &Client,
    bucket: &str,
    s3_key: &str,
    path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let path = path.as_ref();
    let content_type = content_type_for_path(path);
    let body = ByteStream::from_path(path).await?;

    upload_body_to_s3(client_s3, bucket, s3_key, body, content_type).await
}

pub async fn upload_to_s3_from_bytes(
    client_s3: &Client,
    bucket: &str,
    s3_key: &str,
    bytes: Vec<u8>,
    content_type: &'static str,
) -> anyhow::Result<()> {
    let body = ByteStream::from(bytes);
    upload_body_to_s3(client_s3, bucket, s3_key, body, content_type).await
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
