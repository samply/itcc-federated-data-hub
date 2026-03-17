use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibError {
    #[error("FHIR check error")]
    FhirCheckError,

    #[error("Missing fullUrl in bundle entry")]
    MissingFullUrl,

    #[error("Expected Patient resource not found / mismatched id")]
    PatientIdMismatch,

    // If you wrap other errors:
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
