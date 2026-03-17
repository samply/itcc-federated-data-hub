use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use tracing::debug;

pub fn cryptoid_to_fhir_id(cryptoid: &str) -> String {
    let bytes = URL_SAFE_NO_PAD.decode(cryptoid).unwrap();
    debug!("Raw bytes length: {}", bytes.len());
    hex::encode(bytes)
}
pub fn fhir_id_to_cryptoid(fhir_id: &str) -> String {
    let bytes = hex::decode(fhir_id).unwrap();
    URL_SAFE_NO_PAD.encode(&bytes)
}

pub fn cryptoid_to_fhir_id_simple(cryptoid: &str) -> String {
    cryptoid.replace('_', ".")
}
pub fn fhir_id_to_cryptoid_simple(fhir_id: &str) -> String {
    fhir_id.replace('.', "_")
}
