use crate::utils::api_response::ErrorType;
use crate::AppState;
use axum::extract::{Request, State};
use axum::middleware;
use axum::response::IntoResponse;

pub async fn api_key_check(
    State(state): State<AppState>,
    req: Request,
    next: middleware::Next,
) -> Result<impl IntoResponse, ErrorType> {
    // Check header
    let api_key = req
        .headers()
        .get("ApiKey")
        .and_then(|v| v.to_str().ok())
        .ok_or(ErrorType::ApiKeyError)?;

    // Check whether key exists in config
    if api_key == state.api_key {
        Ok(next.run(req).await)
    } else {
        Err(ErrorType::ApiKeyError)
    }
}
