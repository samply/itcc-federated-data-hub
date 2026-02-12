use crate::data::process_maf_object_to_parquet_and_cbio;
use crate::s3::save_files_s3;
use crate::{BEAM_CLIENT, DATALAKE_CONFIG};
use anyhow::{anyhow, Context};
use beam_lib::SocketTask;
use itcc_omics_lib::{FileMeta, MetaData};
use tokio::io::AsyncRead;
use tracing::{error, info};

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
    save_files_s3(&DATALAKE_CONFIG.s3_bucket, incoming, &file_path).await?;
    process_maf_object_to_parquet_and_cbio(&DATALAKE_CONFIG.s3_bucket, &file_path, meta).await?;
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
