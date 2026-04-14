use crate::beam;
use crate::beam::maf_key_from_bytes;
use crate::data::compression::compress_zstd;
use crate::data::transfer::{read_validate_scan, sanitize_maf_bytes};
use crate::pseudonym::build_pseudo_map;
use crate::utils::error_type::ErrorType;
use crate::AppState;
use axum::extract::{DefaultBodyLimit, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, routing::post, Json, Router};
use itcc_omics_lib::beam::Ack;
use itcc_omics_lib::MetaData;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{error, info, info_span};

pub fn routers() -> Router<Arc<AppState>> {
    Router::new()
        .route("/upload/omics", post(upload_handler))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        status_code = tracing::field::Empty,
                    )
                })
                .on_response(
                    |response: &axum::response::Response,
                     latency: std::time::Duration,
                     span: &tracing::Span| {
                        span.record("status_code", response.status().as_u16());
                        info!(parent: span, latency_ms = latency.as_millis(), "response sent");
                    },
                ),
        )
}

// POST /upload/omics
#[tracing::instrument(skip(state, body), fields(file_sha, pseudo_sha, partner_id))]
async fn upload_handler(State(state): State<Arc<AppState>>, body: axum::body::Bytes) -> Response {
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
    drop(body);

    let compressed_vec = match compress_zstd(&psy_res, &state.zstd_level) {
        Ok(v) => v,
        Err(e) => return e.into_response(),
    };
    let pseudo_file_sha = maf_key_from_bytes(&psy_res);
    drop(psy_res);

    let filename = format!("{pseudo_file_sha}.maf.zstd");
    let meta_data = MetaData {
        maf_id: pseudo_file_sha.clone(),
        origin_maf_id: file_sha.clone(),
        partner_id: state.partner_id.clone().to_string(),
        checked_fhir: true,
    };
    let success = SuccessMAF {
        data_warehouse_file_id: pseudo_file_sha,
        local_file_sha: file_sha,
        uploaded_samples: r,
    };
    let send_result = if state.services.enable_sockets {
        let compressed = bytes::Bytes::from(compressed_vec);
        beam::send_file_via_sockets(&state, Some(filename), meta_data, &compressed).await
    } else {
        beam::send_file_via_task(&state, Some(filename), meta_data, &compressed_vec).await
    };

    match send_result {
        Ok(None) => (StatusCode::CREATED, Json(success)).into_response(),
        Ok(Some(msg)) => {
            info!("Upload success: {:?}", msg);
            (StatusCode::CREATED, Json(success)).into_response()
        }
        Err(e) => {
            error!("Error sending file to beam: {:?}", e);
            e.into_response()
        }
    }
}

#[derive(Deserialize, Debug, Clone, Serialize)]
struct SuccessMAF {
    pub data_warehouse_file_id: String,
    pub local_file_sha: String,
    pub uploaded_samples: HashMap<String, String>,
}
