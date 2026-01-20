use crate::omics_data::validator;
use crate::utils::error_type::ErrorType;
use crate::AppState;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{body::Bytes, extract::State, http::HeaderMap, routing::post, Router};
use csv::ReaderBuilder;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs as tokio_fs;
use tracing::{debug, error};

pub fn routers() -> Router<AppState> {
    Router::new()
        .route("/omics/upload", post(upload_handler))
        .layer(DefaultBodyLimit::max(200 * 1024 * 1024))
}

// POST /omics/upload
async fn upload_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
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

    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .comment(Some(b'#'))
        .has_headers(true)
        .flexible(false)
        .from_reader(Cursor::new(body.as_ref()));

    let header = match rdr.headers() {
        Ok(h) => h.clone(),
        Err(e) => {
            tracing::error!("Failed to read headers: {e}");
            return ErrorType::MafEmptyHeader.into_response();
        }
    };
    if let Err(e) = validator::schema_validate(&header, &state.required_omics_columns) {
        return e.into_response();
    }
    match tokio_fs::write(&path, &body).await {
        Ok(_) => (StatusCode::CREATED, format!("stored_as: {filename}")).into_response(),
        Err(e) => {
            error!("Failed to write file {}: {}", path.display(), e);
            ErrorType::WriteFile.into_response()
        }
    }
}
