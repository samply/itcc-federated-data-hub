use crate::beam;
use crate::utils::error_type::ErrorType;
use crate::AppState;
use axum::extract::{Path, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, routing::post, Json, Router};
use itcc_omics_lib::fhir::blaze::{
    get_all_patient_count, get_all_patient_identifiers, get_patient_by_id,
};
use itcc_omics_lib::mainzelliste::handler::CreateTokenResp;
use itcc_omics_lib::mainzelliste::{encryption_ml, init_mainzelliste};
use itcc_omics_lib::patient_id::{PatientId, SampleId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::debug;
use tracing::{error, info, info_span};

pub fn routers() -> Router<Arc<AppState>> {
    Router::new()
        .route("/upload/patient/{id}", post(upload_patient_by_id_handler))
        .route("/upload/patient", post(upload_all_patients_handler))
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

// POST /upload/patient/{id}
#[tracing::instrument(skip(state), fields(patient_id = %id))]
async fn upload_patient_by_id_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    match export_patients_to_dwh(&state, Some(id.clone())).await {
        Ok(result) => (StatusCode::CREATED, Json(result)).into_response(),
        Err(e) => {
            error!("Error exporting patient {}: {:?}", id, e);
            e.into_response()
        }
    }
}

// POST /upload/patient
#[tracing::instrument(skip(state))]
async fn upload_all_patients_handler(State(state): State<Arc<AppState>>) -> Response {
    match export_patients_to_dwh(&state, None).await {
        Ok(result) => (StatusCode::CREATED, Json(result)).into_response(),
        Err(e) => {
            error!("Error exporting all patients: {:?}", e);
            e.into_response()
        }
    }
}

async fn export_patients_to_dwh(
    state: &Arc<AppState>,
    patient_id: Option<String>,
) -> Result<PatientExportResponse, ErrorType> {
    match patient_id {
        Some(id) => {
            let patient_id = PatientId::new(id);
            let patient_vec: HashSet<PatientId> = HashSet::from([patient_id]);
            export_single_patient(state, &patient_vec).await
        }
        None => export_all_patients(state).await,
    }
}

async fn export_single_patient(
    app_state: &Arc<AppState>,
    patient_id: &HashSet<PatientId>,
) -> Result<PatientExportResponse, ErrorType> {
    let token: CreateTokenResp = init_mainzelliste(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
        patient_id.len(),
    )
    .await?;

    let local_crypto_ids = encryption_ml(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
        &token.id,
        patient_id,
    )
    .await?;

    for (patient_id, pseudo_id) in local_crypto_ids.iter() {
        let mut bundle =
            get_patient_by_id(&app_state.http, &app_state.services.blaze_url, patient_id).await?;
        bundle.rename_patient_id_everywhere(patient_id, pseudo_id)?;
        debug!("Bundle AFTER: {:#?}", bundle);
        beam::send_fhir_bundle(app_state, bundle).await?;
    }

    Ok(PatientExportResponse {
        message: format!("Patient {:?} exported successfully", patient_id),
        exported_patient_id: patient_id.to_owned(),
        exported_all: false,
    })
}

async fn export_all_patients(
    app_state: &Arc<AppState>,
) -> Result<PatientExportResponse, ErrorType> {
    let patient_ids: HashSet<PatientId> =
        get_all_patient_identifiers(&app_state.http, &app_state.services.blaze_url, 2).await?; // test todo!
    debug!("Exporting all {:?} patients", patient_ids);
    let token: CreateTokenResp = init_mainzelliste(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
        patient_ids.len(),
    )
    .await?;

    let local_crypto_ids: HashMap<PatientId, PatientId> = encryption_ml(
        &app_state.http,
        app_state.services.ml_api_key.as_ref(),
        &app_state.services.ml_url,
        &token.id,
        &patient_ids,
    )
    .await?;

    for (patient_id, pseudo_id) in local_crypto_ids.iter() {
        let mut bundle =
            get_patient_by_id(&app_state.http, &app_state.services.blaze_url, patient_id).await?;

        bundle.rename_patient_id_everywhere(patient_id, pseudo_id)?;
        debug!("Bundle AFTER: {:#?}", bundle);
        beam::send_fhir_bundle(app_state, bundle).await?;
    }

    Ok(PatientExportResponse {
        message: format!("All {} patients exported successfully", patient_ids.len()),
        exported_patient_id: patient_ids,
        exported_all: true,
    })
}

#[derive(Deserialize, Debug, Clone, Serialize)]
struct PatientExportResponse {
    pub message: String,
    pub exported_patient_id: HashSet<PatientId>,
    pub exported_all: bool,
}
