use crate::utils::error_type::ErrorType;
use crate::AppState;
use axum::extract::{Path, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
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
            info!("Exporting single patient to data warehouse: {}", id);

            // TODO:
            // 1. fetch only this patient
            // 2. transform/export it
            // 3. send it to the data warehouse

            Ok(PatientExportResponse {
                message: format!("Patient {} exported successfully", id),
                exported_patient_id: Some(id),
                exported_all: false,
            })
        }
        None => {
            info!("Exporting all patients to data warehouse");

            // TODO:
            // 1. fetch all patients
            // 2. transform/export them
            // 3. send them to the data warehouse

            Ok(PatientExportResponse {
                message: "All patients exported successfully".to_string(),
                exported_patient_id: None,
                exported_all: true,
            })
        }
    }
}

#[derive(Deserialize, Debug, Clone, Serialize)]
struct PatientExportResponse {
    pub message: String,
    pub exported_patient_id: Option<String>,
    pub exported_all: bool,
}