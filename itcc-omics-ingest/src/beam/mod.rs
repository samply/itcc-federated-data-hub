use crate::utils::config::FileMeta;
use crate::utils::error_type::ErrorType;
use crate::utils::error_type::ErrorType::{BeamError, BeamStreamFileError};
use crate::BEAM_CLIENT;
use axum::body::Bytes;
use beam_lib::AppId;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tracing::error;

pub async fn send_file(
    data_lake_id: AppId,
    suggested_name: Option<String>,
    body: &Bytes,
) -> Result<(), ErrorType> {
    let mut conn = BEAM_CLIENT
        .create_socket_with_metadata(
            &data_lake_id,
            FileMeta {
                suggested_name,
                meta: None,
            },
        )
        .await
        .map_err(|e| {
            error!("Failed to tunnel request: {e}");
            BeamError
        })?;
    conn.write_all(&body).await.map_err(|e| {
        error!("Failed to tunnel response: {e}");
        BeamStreamFileError
    })?;
    Ok(())
}

pub fn maf_key_from_bytes(bytes: &[u8], partner_id: &str) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let sha = hex::encode(h.finalize());
    format!("{partner_id}/{sha}.maf.zstd")
}
