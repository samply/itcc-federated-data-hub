use crate::beam;
use crate::beam::maf_key_from_bytes;
use crate::fhir::handler::get_patient_by_id;
use crate::omics_data::compression::compress_zstd;
use crate::omics_data::transfer::{filter_patient_id, read_validate_scan, sanitize_maf_bytes};
use crate::pseudonym::build_pseudo_map;
use crate::utils::error_type::ErrorType::BeamError;
use crate::AppState;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, http::HeaderMap, routing::post, Router};
use itcc_omics_lib::MetaData;
use tracing::{error, info};

pub fn routers() -> Router<AppState> {
    Router::new()
        .route("/omics/upload", post(upload_handler))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
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
    let compressed = bytes::Bytes::from(compressed_vec.clone());
    let file_sha = maf_key_from_bytes(&psy_res);
    let filename = format!("{file_sha}.maf.zstd");
    let meta_data = MetaData {
        maf_id: file_sha.clone(),
        partner_id: state.partner_id.clone().to_string(),
        checked_fhir: true,
    };
    if let Err(e) =
        beam::send_file_via_task(&state, Some(filename), meta_data, &compressed_vec).await
    {
        return e.into_response();
    }

    (
        StatusCode::CREATED,
        format!("stored_as: {file_sha}.maf.zstd"),
    )
        .into_response()
}
