use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibError {
    #[error("FHIR check error")]
    FhirCheckError,
    #[error("Missing fullUrl in bundle entry")]
    MissingFullUrl,
    #[error("Expected Patient resource not found / mismatched id")]
    PatientIdMismatch,
    #[error("Blaze communication error")]
    BlazeError,
    #[error("Patient not Found")]
    FhirPatientNotFound,
    #[error("Mainzelliste communication error")]
    MlSessionError,
    #[error("Mainzelliste token error")]
    MlTokenError,
    #[error("Mainzelliste error creating patient")]
    MLCreatePatientError,
    #[error("Mainzelliste pseudonym error")]
    PseudoError,
    // If you wrap other errors:
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
