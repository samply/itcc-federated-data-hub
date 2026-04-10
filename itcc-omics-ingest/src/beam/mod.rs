use crate::utils::config::AppState;
use crate::utils::error_type::ErrorType;
use crate::utils::error_type::ErrorType::{BeamError, BeamStreamFileError};
use axum::body::Bytes;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use beam_lib::{BlockingOptions, MsgId, TaskRequest};
use itcc_omics_lib::beam::{Ack, FileMeta};
use itcc_omics_lib::fhir::bundle::Bundle;
use itcc_omics_lib::fhir::IngestTask;
use itcc_omics_lib::{MafTask, MetaData};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};

pub async fn send_fhir_bundle(state: &AppState, bundle: Bundle) -> Result<Vec<Ack>, ErrorType> {
    let task = TaskRequest {
        id: MsgId::new(),
        from: state.services.beam_id.clone(),
        to: vec![state.services.dwh_task_id.clone()],
        body: vec![IngestTask::Fhir { bundle }],
        ttl: "60s".to_string(),
        failure_strategy: beam_lib::FailureStrategy::Discard,
        metadata: ().try_into().unwrap(),
    };

    state.beam_client.post_task(&task).await.map_err(|e| {
        error!("Failed to task request: {e}");
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

pub async fn send_file_via_sockets(
    state: &AppState,
    suggested_name: Option<String>,
    meta_data: MetaData,
    body: &Bytes,
) -> Result<Option<Vec<Ack>>, ErrorType> {
    let meta_json = serde_json::to_value(&meta_data).map_err(|_| ErrorType::BeamError)?;
    let mut conn = state
        .beam_client
        .create_socket_with_metadata(
            &state.services.dwh_socket_id,
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
    Ok(None)
}

pub async fn send_file_via_task(
    state: &AppState,
    suggested_name: Option<String>,
    meta_data: MetaData,
    body: &[u8],
) -> Result<Option<Vec<Ack>>, ErrorType> {
    let task_body = MafTask {
        meta: meta_data,
        suggested_name,
        bytes_b64: STANDARD.encode(body),
    };
    let task = TaskRequest {
        id: MsgId::new(),
        from: state.services.beam_id.clone(),
        to: vec![state.services.dwh_task_id.clone()],
        body: vec![IngestTask::Maf(task_body)],
        ttl: "60s".to_string(),
        failure_strategy: beam_lib::FailureStrategy::Discard,
        metadata: ().try_into().unwrap(),
    };
    info!("Posting MAF task id={} to={:?}", task.id, task.to);

    state.beam_client.post_task(&task).await.map_err(|e| {
        error!("Failed to task request: {e}");
        BeamError
    })?;

    let results = state
        .beam_client
        .poll_results::<Vec<Ack>>(&task.id, &BlockingOptions::from_count(1))
        .await
        .map_err(|e| {
            error!("Failed to tunnel request: {e}");
            BeamError
        })?
        .into_iter()
        .map(|r| r.body)
        .flatten()
        .collect();

    Ok(Some(results))
}

pub fn maf_key_from_bytes(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}
