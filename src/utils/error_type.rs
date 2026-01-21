use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum ErrorType {
    WriteFile,
    NonEmptyDir,
    ApiKeyError,
    MafEmptyHeader,
    MafDuplicateHeader,
    MafMissingColumn,
    BeamError,
    BeamStreamFileError,
}
impl IntoResponse for ErrorType {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ErrorType::NonEmptyDir => (
                StatusCode::NOT_FOUND,
                "Directory already exists and is non-empty".to_string(),
            ),
            ErrorType::WriteFile => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to write file on server".to_string(),
            ),
            ErrorType::ApiKeyError => {
                (StatusCode::UNAUTHORIZED, "API key is not valid".to_string())
            }
            ErrorType::MafEmptyHeader => {
                (StatusCode::BAD_REQUEST, "Header is malformed".to_string())
            }
            ErrorType::MafDuplicateHeader => {
                (StatusCode::CONFLICT, "Header already exists".to_string())
            }
            ErrorType::MafMissingColumn => {
                (StatusCode::BAD_REQUEST, "Column is missing".to_string())
            }
            ErrorType::BeamError => (StatusCode::INTERNAL_SERVER_ERROR, "beam error".to_string()),
            ErrorType::BeamStreamFileError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "beam stream error".to_string(),
            ),
        };
        (status, Json(body)).into_response()
    }
}
