use crate::AppState;
use axum::response::IntoResponse;
use axum::{http::StatusCode, middleware};
use axum::extract::{Request, State};

pub async fn api_key_check(
    State(state): State<AppState>,
    req: Request,
    next: middleware::Next,
) -> Result<impl IntoResponse, StatusCode> {
    // Check header
    let api_key = req
        .headers()
        .get("ApiKey")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check whether key exists in config
    if api_key == state.api_key {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
