use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs as tokio_fs;
use tracing::error;

use crate::AppState;

#[derive(Serialize)]
struct UploadResponse {
    status: String,
    stored_as: String,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/omics/health", get(health))
        .route("/omics/upload", post(upload_handler))
        .with_state(state)
}

// GET /omics/healthz
async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

// POST /omics/upload
async fn upload_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
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

    if let Err(e) = tokio_fs::write(&path, &body).await {
        error!("Failed to write file {}: {}", path.display(), e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to write file on server",
        )
            .into_response();
    }

    let resp = UploadResponse {
        status: "ok".to_string(),
        stored_as: filename,
    };

    (StatusCode::OK, Json(resp)).into_response()
}
