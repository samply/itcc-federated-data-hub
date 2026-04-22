use crate::beam;
use crate::beam::maf_key_from_bytes;
use crate::data::compression::compress_zstd;
use crate::data::transfer::{read_validate_scan, sanitize_maf_bytes};
use crate::pseudonym::{build_pseudo_map, fhir_collector_sender, run_pseudonymisation};
use crate::utils::error_type::ErrorType;
use crate::AppState;
use axum::extract::{DefaultBodyLimit, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, routing::post, Json, Router};
use itcc_omics_lib::beam::Ack;
use itcc_omics_lib::patient_id::{patient_grouped_sample_id, PatientId, SampleId};
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
async fn upload_handler(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> Result<Response, ErrorType> {
    let file_sha = maf_key_from_bytes(body.as_ref());

    let sample_ids: HashSet<SampleId> = read_validate_scan(&body, &state).await?;
    info!("Upload scan: {:?}", sample_ids);

    // Mainzelliste collecting all pseudonyms exec encryption
    let local_pseudonym_ids: HashMap<PatientId, PatientId> =
        run_pseudonymisation(&state, &sample_ids).await?;
    // fhir handling
    fhir_collector_sender(&state, &local_pseudonym_ids).await?;
    // map pseudonym {sample_id}_suffix to {pseudonym}_suffix
    let sample_ids_pseudonym_sample_ids: HashMap<SampleId, SampleId> =
        build_pseudo_map(&sample_ids, &local_pseudonym_ids).await?;

    let pseudomyn_maf = sanitize_maf_bytes(&body, &sample_ids_pseudonym_sample_ids)?;
    drop(body);

    let compressed_maf = compress_zstd(&pseudomyn_maf, &state.zstd_level)?;
    // identifier as hash
    let pseudo_file_sha = maf_key_from_bytes(&pseudomyn_maf);
    drop(pseudomyn_maf);

    let filename = format!("{pseudo_file_sha}.maf.zstd");
    let patient_to_sample: HashMap<PatientId, HashSet<SampleId>> =
        patient_grouped_sample_id(&sample_ids_pseudonym_sample_ids, &local_pseudonym_ids);

    let meta_data = MetaData {
        maf_id: pseudo_file_sha.clone(),
        origin_maf_id: file_sha.clone(),
        partner_id: state.partner_id.clone().to_string(),
        checked_fhir: true,
        patient_sample_suffix: true,
        patient_to_sample,
    };

    let send_result = if state.services.enable_sockets {
        let compressed = bytes::Bytes::from(compressed_maf);
        beam::send_file_via_sockets(&state, Some(filename), meta_data, &compressed).await?
    } else {
        beam::send_file_via_task(&state, Some(filename), meta_data, &compressed_maf).await?
    };

    let success = SuccessMAF {
        data_warehouse_file_id: pseudo_file_sha,
        local_file_sha: file_sha,
        patient_id_to_pseudonym_id: local_pseudonym_ids,
    };
    match send_result {
        None => Ok((StatusCode::CREATED, Json(success)).into_response()),
        Some(msg) => {
            info!("Upload success: {:?}", msg);
            Ok((StatusCode::CREATED, Json(success)).into_response())
        }
    }
}

#[derive(Deserialize, Debug, Clone, Serialize)]
struct SuccessMAF {
    pub data_warehouse_file_id: String,
    pub local_file_sha: String,
    pub patient_id_to_pseudonym_id: HashMap<PatientId, PatientId>,
}
