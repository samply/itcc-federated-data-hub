use crate::utils::error_type::ErrorType;
use crate::AppState;
use crate::beam;
use tracing::debug;
use axum::extract::{Path, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{error, info, info_span};
use itcc_omics_lib::mainzelliste::handler::{
create_patients, create_session, create_token, CreatePatientResp, CreateTokenResp,
};
use itcc_omics_lib::mainzelliste::{encryption_ml, init_mainzelliste};
use itcc_omics_lib::fhir::blaze::{
    get_all_patient_count,
    get_all_patient_identifiers,
    get_patient_by_id,
};

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
async fn upload_all_patients_handler(
    State(state): State<Arc<AppState>>,
) -> Response {
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
            let patient_vec: HashSet<String> = HashSet::from([id]);
            export_single_patient(state, patient_vec).await
        },
        None => export_all_patients(state).await,
    }
}

async fn export_single_patient(
    app_state: &Arc<AppState>,
    patient_id: HashSet<String>,
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
        patient_id.clone(),   
    )
    .await?;

    for (patient_id, pseudo_id) in local_crypto_ids.iter() {

            let mut bundle = get_patient_by_id(
                &app_state.http,
                &app_state.services.blaze_url,
                patient_id,
            ).await?;
            bundle.rename_patient_id_everywhere(patient_id, pseudo_id)?;
            debug!("Bundle AFTER: {:#?}", bundle);
            beam::send_fhir_bundle(app_state, bundle).await?;
    }

    let exported_id = patient_id.iter().next().cloned().unwrap_or_default();
 
    Ok(PatientExportResponse {
        message: format!("Patient {} exported successfully", exported_id),
        exported_patient_id: patient_id,
        exported_all: false,
    })
}

async fn export_all_patients(
    state: &Arc<AppState>,
) -> Result<PatientExportResponse, ErrorType> {
    let count = get_all_patient_count(
        &state.http,
        &state.services.blaze_url,
    ).await?;

    let patient_ids = get_all_patient_identifiers(
        &state.http,
        &state.services.blaze_url,
        count,
    ).await?;

    for patient_id in patient_ids {
        let mut bundle = get_patient_by_id(
            &state.http,
            &state.services.blaze_url,
            &patient_id,
        ).await?;

        let pseudo_id = patient_id.clone(); // TEMP ONLY

        bundle.rename_patient_id_everywhere(&patient_id, &pseudo_id)?;
        debug!("Bundle: {:#?}", bundle);
        beam::send_fhir_bundle(state, bundle).await?;
    }

    Ok(PatientExportResponse {
        message: "All patients exported successfully".to_string(),
        exported_patient_id: HashSet::new(),
        exported_all: true,
    })
}

#[derive(Deserialize, Debug, Clone, Serialize)]
struct PatientExportResponse {
    pub message: String,
    pub exported_patient_id: HashSet<String>,
    pub exported_all: bool,
}