use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use itcc_omics_lib::error_type::LibError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum ErrorType {
    WriteFile,
    CompressFile,
    NonEmptyDir,
    ApiKeyError,
    MafEmptyHeader,
    MafDuplicateHeader,
    MafMissingColumn,
    CsvError,
    BeamError,
    BeamStreamFileError,
    PseudoError,
    MlSessionError,
    MlTokenError,
    MLCreatePatientError,
    MafWriteError,
    BlazeError,
    BlazeResultError,
    FhirCheckError,
    FhirPatientNotFound,
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
            ErrorType::CompressFile => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "compression error".to_string(),
            ),
            ErrorType::CsvError => (StatusCode::INTERNAL_SERVER_ERROR, "csv error".to_string()),
            ErrorType::PseudoError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "error by providing pseudomiesation".to_string(),
            ),
            ErrorType::MafWriteError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "error writing MAF".to_string(),
            ),
            ErrorType::MlSessionError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "mainzelliste session error".to_string(),
            ),
            ErrorType::MlTokenError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "mainzelliste token error".to_string(),
            ),
            ErrorType::MLCreatePatientError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "mainzelliste patient creation error".to_string(),
            ),
            ErrorType::BlazeError => (StatusCode::INTERNAL_SERVER_ERROR, "blaze error".to_string()),
            ErrorType::FhirCheckError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "fhir check error".to_string(),
            ),
            ErrorType::FhirPatientNotFound => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "fhir data for patient not found please provide".to_string(),
            ),
            ErrorType::BlazeResultError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "result error".to_string(),
            ),
        };
        (status, Json(body)).into_response()
    }
}

impl From<LibError> for ErrorType {
    fn from(e: LibError) -> Self {
        match e {
            LibError::FhirCheckError | LibError::MissingFullUrl | LibError::PatientIdMismatch => {
                ErrorType::FhirCheckError
            }
            LibError::Other(_) => ErrorType::BlazeError,
            LibError::BlazeError => ErrorType::BlazeError,
            LibError::FhirPatientNotFound => ErrorType::FhirPatientNotFound,
            LibError::MlSessionError => ErrorType::MlSessionError,
            LibError::MlTokenError => ErrorType::MlTokenError,
            LibError::MLCreatePatientError => ErrorType::MLCreatePatientError,
            LibError::PseudoError => ErrorType::PseudoError,
            LibError::BlazeResultError => ErrorType::BlazeResultError,
        }
    }
}
