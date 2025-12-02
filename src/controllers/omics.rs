use axum::{body::Bytes, extract::State, http::{HeaderMap}, routing::post, Router};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs as tokio_fs;
use tracing::error;
use crate::AppState;
use crate::utils::api_response::{ApiResult, ErrorType, SuccessType};

pub fn routers() -> Router<AppState> {
    Router::new()
        .route("/omics/upload", post(upload_handler))
}


// POST /omics/upload
async fn upload_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes
) -> ApiResult {
    
    let header_name = headers
        .get("x-filename")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("omics_payload.bin");

    let sanitized_name = Path::new(header_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("omics_payload.bin");

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let filename = format!("{ts}_{sanitized_name}");

    let mut path: PathBuf = (*state.upload_dir).clone();
    path.push(&filename);

    match tokio_fs::write(&path, &body).await {
        Ok(_) => Ok(SuccessType::UploadResponse(filename)),
        Err(e) => {
            error!("Failed to write file {}: {}", path.display(), e);
            Err(ErrorType::WriteFile)
        }
    }
}
