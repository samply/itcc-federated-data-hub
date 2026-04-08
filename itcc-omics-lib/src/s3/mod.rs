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

pub async fn upload_to_s3_from_path(
    client_s3: &Client,
    bucket: &str,
    s3_key: &str,
    path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let path = path.as_ref();
    let content_type = content_type_for_path(path);

    debug!("uploading to s3 key={s3_key} content_type={content_type}");

    let bytes = tokio::fs::read(path).await?;
    let len = bytes.len() as i64;
    let body = ByteStream::from(bytes);

    client_s3
        .put_object()
        .bucket(bucket)
        .key(s3_key)
        .content_type(content_type)
        .content_length(len)
        .body(body)
        .send()
        .await?;

    info!("s3 {s3_key} saved");
    Ok(())
}
pub async fn upload_to_s3_from_bytes(
    client_s3: &Client,
    bucket: &str,
    s3_key: &str,
    bytes: Vec<u8>,
    content_type: &'static str,
) -> anyhow::Result<()> {
    let len = bytes.len() as i64;
    let body = ByteStream::from(bytes);

    debug!("uploading to s3 key={s3_key} content_type={content_type}");

    client_s3
        .put_object()
        .bucket(bucket)
        .key(s3_key)
        .content_type(content_type)
        .content_length(len)
        .body(body)
        .send()
        .await?;

    info!("s3 {s3_key} saved");
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

    let bytes = resp.body.collect().await?.into_bytes();

    let tmp = NamedTempFile::new()?;
    let (_file, path) = tmp.keep()?;

    tokio::fs::write(&path, &bytes).await?;

    Ok(path)
}
// Check if a file (key) exists in the S3 bucket.
// Returns `true` if it exists, `false` if not found.
pub async fn key_exists(client_s3: &Client, bucket: &str, s3_key: &str) -> anyhow::Result<bool> {
    let result = client_s3
        .head_object()
        .bucket(bucket)
        .key(s3_key)
        .send()
        .await;

    match result {
        Ok(_) => Ok(true),
        Err(e) => {
            // SdkError wraps a service error — check if it's a 404 Not Found
            let is_not_found = e
                .as_service_error()
                .map(|se| se.is_not_found())
                .unwrap_or(false);

            if is_not_found {
                Ok(false)
            } else {
                Err(anyhow::anyhow!(e)) // Bubble up real errors
            }
        }
    }
}

// List all keys in a bucket under an optional prefix (folder).
// Pass `prefix: None` to list everything, or `Some("folder/")` to scope it.
pub async fn list_keys(
    client_s3: &Client,
    bucket: &str,
    prefix: Option<&str>,
) -> anyhow::Result<Vec<String>> {
    let mut keys = Vec::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let mut req = client_s3.list_objects_v2().bucket(bucket);

        if let Some(p) = prefix {
            req = req.prefix(p);
        }
        if let Some(token) = continuation_token {
            req = req.continuation_token(token);
        }

        let resp = req.send().await?;

        for obj in resp.contents() {
            if let Some(key) = obj.key() {
                keys.push(key.to_string());
            }
        }

        if resp.is_truncated().unwrap_or(false) {
            continuation_token = resp.next_continuation_token().map(str::to_string);
        } else {
            break;
        }
    }

    info!(
        "listed {} keys in s3://{}/{}",
        keys.len(),
        bucket,
        prefix.unwrap_or("")
    );
    Ok(keys)
}

pub async fn copy_s3_object(
    client_s3: &Client,
    source_bucket: &str,
    source_key: &str,
    dest_bucket: &str,
    dest_key: &str,
) -> anyhow::Result<()> {
    let copy_source = format!("{}/{}", source_bucket, source_key);

    client_s3
        .copy_object()
        .copy_source(&copy_source)
        .bucket(dest_bucket)
        .key(dest_key)
        .send()
        .await?;
    info!("s3 {dest_bucket}={dest_bucket}");
    Ok(())
}
