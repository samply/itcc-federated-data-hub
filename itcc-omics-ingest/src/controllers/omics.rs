use crate::beam;
use crate::beam::maf_key_from_bytes;
use crate::omics_data::compression::compress_zstd;
use crate::omics_data::transfer::{filter_patient_id, read_validate_scan, sanitize_maf_bytes};
use crate::pseudonym::build_pseudo_map;
use crate::AppState;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, http::HeaderMap, routing::post, Router};
use tracing::{error, info};

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

    let res = match read_validate_scan(&body, &state).await {
        Ok(v) => v,
        Err(e) => return e.into_response(),
    };
    info!("Upload scan: {:?}", res);

    let r = match build_pseudo_map(&state, res).await {
        Ok(v) => v,
        Err(e) => return e.into_response(),
    };
    let psy_res = match sanitize_maf_bytes(&body, &r) {
        Ok(v) => v,
        Err(e) => return e.into_response(),
    };

    let compressed_vec = match compress_zstd(&psy_res, &state.zstd_level) {
        Ok(v) => v,
        Err(e) => return e.into_response(),
    };
    let compressed = bytes::Bytes::from(compressed_vec);

    let filename = maf_key_from_bytes(&psy_res, &state.partner_id);
    if let Err(e) = beam::send_file(state.data_lake_id, Some(filename.clone()), &compressed).await {
        return e.into_response();
    }

    (StatusCode::CREATED, format!("stored_as: {filename}")).into_response()
}
