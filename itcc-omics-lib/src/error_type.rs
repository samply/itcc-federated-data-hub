use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibError {
    #[error("FHIR check error")]
    FhirCheckError,
    #[error("sample id must not be empty")]
    SampleIdEmpty,
    #[error("sample id must follow '<base>_<suffix>' format, got: '{0}'")]
    SampleIdInvalidFormat(String),
    #[error("Missing fullUrl in bundle entry")]
    MissingFullUrl,
    #[error("Expected Patient resource not found / mismatched id")]
    PatientIdMismatch,
    #[error("Blaze error to much patients")]
    BlazeResultError,
    #[error("Blaze communication error")]
    BlazeError,
    #[error("Blaze connection error for URL {url}: {message}")]
    BlazeConnectionError { url: String, message: String },
    #[error("Blaze parse error: {0}")]
    BlazeParseError(String),
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
    #[error("Meta data conflict")]
    MetaDataError,
    // If you wrap other errors:
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
