use crate::beam;
use crate::beam::maf_key_from_bytes;
use crate::data::compression::compress_zstd;
use crate::data::transfer::{read_validate_scan, sanitize_maf_bytes};
use crate::pseudonym::build_pseudo_map;
use crate::AppState;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, http::HeaderMap, routing::post, Router};
use itcc_omics_lib::beam::MetaData;
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
    let file_sha = maf_key_from_bytes(body.as_ref());

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
    let pseudo_file_sha = maf_key_from_bytes(&psy_res);
    let filename = format!("{pseudo_file_sha}.maf.zstd");
    let meta_data = MetaData {
        maf_id: pseudo_file_sha.clone(),
        partner_id: state.partner_id.clone().to_string(),
        checked_fhir: true,
    };
    if let Err(e) =
        beam::send_file_via_task(&state, Some(filename), meta_data, &compressed_vec).await
    {
        return e.into_response();
    }

    (StatusCode::CREATED, format!("stored DHW as: {pseudo_file_sha}, (Optional)file sha256(provided maf file): {file_sha}"))
        .into_response()
}
