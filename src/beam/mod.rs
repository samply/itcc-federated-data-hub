use crate::config::FileMeta;
use crate::utils::error_type::ErrorType;
use crate::utils::error_type::ErrorType::{BeamError, BeamStreamFileError};
use crate::BEAM_CLIENT;
use axum::body::Bytes;
use beam_lib::AppId;
use std::io::Cursor;
use tracing::error;

pub async fn send_file(
    data_lake_id: AppId,
    cancer_study_identifier: &str,
    body: &Bytes,
) -> Result<(), ErrorType> {
    let mut conn = BEAM_CLIENT
        .create_socket_with_metadata(
            &data_lake_id,
            FileMeta {
                suggested_name: None,
                meta: None,
            },
        )
        .await
        .map_err(|e| {
            error!("Failed to tunnel request: {e}");
            BeamError
        })?;
    let mut reader = Cursor::new(body);
    tokio::io::copy(&mut reader, &mut conn).await.map_err(|e| {
        error!("Failed to tunnel response: {e}");
        BeamStreamFileError
    })?;
    Ok(())
}
