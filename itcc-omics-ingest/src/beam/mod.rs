use crate::utils::config::AppState;
use crate::utils::error_type::ErrorType;
use crate::utils::error_type::ErrorType::{BeamError, BeamStreamFileError};
use axum::body::Bytes;
use beam_lib::reqwest::Url as beam_Url;
use beam_lib::{BlockingOptions, MsgId, TaskRequest};
use itcc_omics_lib::fhir::bundle::Bundle;
use itcc_omics_lib::fhir::FhirBundleTask;
use itcc_omics_lib::{Ack, FileMeta, MetaData};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tracing::error;

pub async fn send_fhir_bundle(state: &AppState, bundle: Bundle) -> Result<Vec<Ack>, ErrorType> {
    let task = TaskRequest {
        id: MsgId::new(),
        from: state.services.beam_id.clone(),
        to: vec![state.data_lake_id.clone()],
        body: vec![FhirBundleTask { bundle }],
        ttl: "60s".to_string(),
        failure_strategy: beam_lib::FailureStrategy::Discard,
        metadata: ().try_into().unwrap(),
    };

    state.beam_client.post_task(&task).await.map_err(|e| {
        error!("Failed to tunnel request: {e}");
        BeamError
    })?;

    let results = state
        .beam_client
        .poll_results::<Vec<Ack>>(&task.id, &BlockingOptions::from_count(1))
        .await
        .map_err(|e| {
            error!("Failed to tunnel request: {e}");
            BeamError
        })?;

    Ok(results.into_iter().map(|r| r.body).flatten().collect())
}

pub async fn send_file(
    state: &AppState,
    suggested_name: Option<String>,
    meta_data: MetaData,
    body: &Bytes,
) -> Result<(), ErrorType> {
    let meta_json = serde_json::to_value(&meta_data).map_err(|_| ErrorType::BeamError)?;
    let mut conn = state
        .beam_client
        .create_socket_with_metadata(
            &state.data_lake_id,
            FileMeta {
                suggested_name,
                meta: Some(meta_json),
            },
        )
        .await
        .map_err(|e| {
            error!("Failed to tunnel request: {e}");
            BeamError
        })?;
    conn.write_all(&body).await.map_err(|e| {
        error!("Failed to tunnel response: {e}");
        BeamStreamFileError
    })?;
    Ok(())
}

pub fn maf_key_from_bytes(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}
