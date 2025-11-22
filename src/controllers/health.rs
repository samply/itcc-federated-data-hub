use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use crate::AppState;

pub fn routers() -> Router<AppState> {
    Router::new()
        .route("/omics/health", get(health))
}


// GET /omics/health
async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}
