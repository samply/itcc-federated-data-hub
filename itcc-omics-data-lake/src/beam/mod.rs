use crate::data::{process_maf_object_to_parquet, save_files_s3};
use crate::{BEAM_CLIENT, DATALAKE_CONFIG};
use anyhow::{anyhow, Context};
use beam_lib::{BlockingOptions, SocketTask, TaskRequest, TaskResult, WorkStatus};
use futures::future::join_all;
use itcc_omics_lib::fhir::bundle::Bundle;
use itcc_omics_lib::fhir::FhirBundleTask;
use itcc_omics_lib::s3::client::s3_client;
use itcc_omics_lib::{Ack, FileMeta, MetaData};
use std::time::Duration;
use tokio::io::AsyncRead;
use tracing::{debug, error, info, warn};

pub async fn run_socket_polling() -> anyhow::Result<()> {
    BEAM_CLIENT
        .handle_sockets(|task, incoming| async move {
            info!("[Beam] Starting socket polling...");
            // print_file(task, incoming).await;
            if let Err(e) = beam_save_generate(task, incoming).await {
                tracing::error!("save_file_as_s3 failed: {e:#}");
            }
            Ok(())
        })
        .await?;
    Ok(())
}

async fn beam_save_generate(
    socket_task: SocketTask,
    mut incoming: impl AsyncRead + Unpin,
) -> anyhow::Result<()> {
    let s3_client: &aws_sdk_s3::Client = s3_client().await;
    let from = socket_task
        .from
        .as_ref()
        .split('.')
        .take(2)
        .collect::<Vec<_>>()
        .join(".");
    let file_meta: FileMeta =
        serde_json::from_value(socket_task.metadata).context("Failed to deserialize metadata")?;
    let suggested_name = file_meta
        .suggested_name
        .clone()
        .ok_or_else(|| anyhow!("Missing suggested_name in FileMeta"))?;
    info!(
        "BEAM send_file metadata = {}",
        serde_json::to_string(&file_meta).unwrap()
    );
    let meta: MetaData = match file_meta.meta {
        Some(v) => serde_json::from_value(v).context("Failed to deserialize MetaData")?,
        None => return Err(anyhow!("Missing meta JSON in FileMeta.meta")),
    };
    info!(
        maf_id = %meta.maf_id,
        partner_id = %meta.partner_id,
        checked_fhir = meta.checked_fhir,
        suggested_name = %suggested_name,
        "[Beam] received file + metadata"
    );
    let file_path = format!("{}/{}", meta.partner_id, suggested_name);
    save_files_s3(s3_client, &DATALAKE_CONFIG.s3_bucket, incoming, &file_path).await?;
    process_maf_object_to_parquet(s3_client, &DATALAKE_CONFIG.s3_bucket, &file_path, meta).await?;
    Ok(())
}

async fn print_file(
    socket_task: SocketTask,
    mut incoming: impl AsyncRead + Unpin,
) -> anyhow::Result<()> {
    info!("Incoming file from {}", socket_task.from);
    tokio::io::copy(&mut incoming, &mut tokio::io::stdout()).await?;
    info!("Done printing file from {}", socket_task.from);
    Ok(())
}

pub async fn run_task_polling() -> anyhow::Result<()> {
    info!("Starting Beam task polling on {}", DATALAKE_CONFIG.beam_url);

    let block_one = BlockingOptions::from_count(1);

    loop {
        match BEAM_CLIENT.poll_pending_tasks(&block_one).await {
            Ok(tasks) => {
                join_all(tasks.into_iter().map(|task| async move {
                    let claimed = TaskResult {
                        from: DATALAKE_CONFIG.beam_id.clone(),
                        to: vec![task.from.clone()],
                        task: task.id.clone(),
                        status: WorkStatus::Claimed,
                        body: (),
                        metadata: ().into(),
                    };

                    if let Err(e) = BEAM_CLIENT.put_result(&claimed, &claimed.task).await {
                        warn!("Failed to claim task from {}: {e}", claimed.to[0]);
                        return;
                    }

                    tokio::spawn(handle_task(task));
                }))
                .await;
            }

            Err(beam_lib::BeamError::ReqwestError(e)) if e.is_connect() => {
                warn!(
                    "Failed to connect to beam proxy on {}. Retrying in 30s",
                    DATALAKE_CONFIG.beam_url
                );
                tokio::time::sleep(Duration::from_secs(30)).await;
            }

            Err(e) => {
                warn!("Error during task polling {e}");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn handle_task(task: TaskRequest<Vec<FhirBundleTask>>) {
    let from = task.from.clone();

    let results: Vec<Ack> = join_all(
        task.body
            .into_iter()
            .map(|t| async move { handle_fhir_bundle(t.bundle).await }),
    )
    .await;

    let put = BEAM_CLIENT
        .put_result(
            &TaskResult {
                from: DATALAKE_CONFIG.beam_id.clone(),
                to: vec![from],
                task: task.id.clone(),
                status: WorkStatus::Succeeded,
                body: results,
                metadata: ().try_into().unwrap(),
            },
            &task.id,
        )
        .await;

    if let Err(e) = put {
        warn!("Failed to respond to task: {e}");
    }
}

async fn handle_fhir_bundle(_bundle: Bundle) -> Ack {
    // store to blaze
    debug!("Received Bundle Task");
    debug!("Beam: {:#?}", _bundle);
    Ack {
        ok: true,
        message: None,
    }
}
