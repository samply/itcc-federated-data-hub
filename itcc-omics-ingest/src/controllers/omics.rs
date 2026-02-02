use crate::beam;
use crate::beam::maf_key_from_bytes;
use crate::omics_data::compression::compress_zstd;
use crate::omics_data::validator;
use crate::utils::error_type::ErrorType;
use crate::AppState;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, http::HeaderMap, routing::post, Router};
use csv::ReaderBuilder;
use std::io::Cursor;
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
    body: axum::body::Bytes,
) -> Response {
    let header_name = headers
        .get("x-filename")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("omics_payload.bin");

    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .comment(Some(b'#'))
        .has_headers(true)
        .flexible(false)
        .from_reader(Cursor::new(&body));

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

    let compressed_vec = match compress_zstd(&body, &state.zstd_level) {
        Ok(v) => v,
        Err(e) => return e.into_response(),
    };
    let compressed = bytes::Bytes::from(compressed_vec);

    let filename = maf_key_from_bytes(&body, &state.partner_id);
    if let Err(e) = beam::send_file(state.data_lake_id, Some(filename.clone()), &compressed).await {
        return e.into_response();
    }

    (StatusCode::CREATED, format!("stored_as: {filename}")).into_response()
}
