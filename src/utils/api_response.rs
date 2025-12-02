use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

// Type for successfull Response with data
pub type ApiResult = Result<SuccessType, ErrorType>;

#[derive(Deserialize, Serialize, Debug)]
pub enum SuccessType {
    Health,
    UploadResponse(String),
}

#[derive(Deserialize, Serialize, Debug)]
pub enum ErrorType {
    WriteFile,
    NonEmptyDir,
    ApiKeyError,
}
impl IntoResponse for ErrorType {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ErrorType::NonEmptyDir => {
                (StatusCode::NOT_FOUND, "Directory already exists and is non-empty".to_string())
            }
            ErrorType::WriteFile => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to write file on server".to_string(),
                )
            }
            ErrorType::ApiKeyError => {
                (StatusCode::UNAUTHORIZED, "API key is not valid".to_string())
            }
        };
        (status, Json(body)).into_response()
    }
}

impl IntoResponse for SuccessType {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            SuccessType::Health => {
                (StatusCode::OK, "OK".to_string())
            }
            SuccessType::UploadResponse(filename) => {
                (StatusCode::CREATED, format!("{filename}"))
            }
        };
        (status, Json(body)).into_response()
    }
}
