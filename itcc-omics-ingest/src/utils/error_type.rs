use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use itcc_omics_lib::error_type::LibError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum ErrorType {
    CompressFile,
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
    BlazeParseError(String),
    BlazeConnectionError(String),
    MafInvalidSampleId(Vec<String>),
}
impl IntoResponse for ErrorType {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ErrorType::ApiKeyError => (
                StatusCode::UNAUTHORIZED,
                "API key is invalid or missing".to_string(),
            ),
            ErrorType::MafEmptyHeader => (
                StatusCode::BAD_REQUEST,
                "MAF header is empty or could not be read".to_string(),
            ),
            ErrorType::MafDuplicateHeader => (
                StatusCode::CONFLICT,
                "MAF file has duplicated headers".to_string(),
            ),
            ErrorType::MafMissingColumn => (
                StatusCode::BAD_REQUEST,
                "MAF file is missing one or more required columns".to_string(),
            ),
            ErrorType::MafInvalidSampleId(ids) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!(
                    "Invalid sample ID format. Expected '{{base}}_{{suffix}}' (e.g. 'SAMPLE001_T1'). Offending IDs: {}", ids.join(", ")
                ),
            ),
            ErrorType::BeamError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to send data via Beam".to_string(),
            ),
            ErrorType::BeamStreamFileError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to stream file via Beam socket".to_string(),
            ),
            ErrorType::MafWriteError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to write pseudonymized MAF file".to_string(),
            ),
            ErrorType::CsvError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse MAF file as CSV".to_string(),
            ),
            ErrorType::CompressFile => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to compress file before transfer".to_string(),
            ),
            ErrorType::PseudoError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Pseudonymization failed".to_string(),
            ),
            ErrorType::MlSessionError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create Mainzelliste session".to_string(),
            ),
            ErrorType::MlTokenError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to obtain Mainzelliste token".to_string(),
            ),
            ErrorType::MLCreatePatientError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create patient in Mainzelliste".to_string(),
            ),
            ErrorType::FhirCheckError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "FHIR resource validation failed".to_string(),
            ),
            ErrorType::FhirPatientNotFound => (
                StatusCode::NOT_FOUND,
                "No FHIR data found for this patient — upload a FHIR bundle first".to_string(),
            ),
            ErrorType::BlazeError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Blaze FHIR server returned an error".to_string(),
            ),
            ErrorType::BlazeConnectionError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to connect to Blaze: {msg}"),
            ),
            ErrorType::BlazeParseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse Blaze response: {msg}"),
            ),
            ErrorType::BlazeResultError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Blaze returned an unexpected or empty result".to_string(),
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
            LibError::BlazeConnectionError { url, message } => {
                ErrorType::BlazeConnectionError(format!("{url}: {message}"))
            }
            LibError::BlazeParseError(msg) => ErrorType::BlazeParseError(msg),
        }
    }
}
