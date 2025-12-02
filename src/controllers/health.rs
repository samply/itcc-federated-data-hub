use crate::utils::api_response::SuccessType;
use crate::AppState;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

pub fn routers() -> Router<AppState> {
    Router::new().route("/omics/health", get(health))
}

// GET /omics/health
async fn health() -> impl IntoResponse {
    SuccessType::Health
}
