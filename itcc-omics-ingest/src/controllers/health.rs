use crate::AppState;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;

pub fn routers() -> Router<Arc<AppState>> {
    Router::new().route("/omics/health", get(health))
}

// GET /omics/health
async fn health() -> impl IntoResponse {
    (StatusCode::OK, "OK".to_string())
}
