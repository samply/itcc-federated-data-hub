use crate::s3::upload_stream_to_s3;
use crate::utils::config::FileMeta;
use crate::{BEAM_CLIENT, CONFIG};
use anyhow::Context;
use aws_sdk_s3::Client;
use beam_lib::{AppId, SocketTask};
use std::path::Path;
use tokio::io::AsyncRead;
use tracing::{error, info};

pub async fn run_socket_polling() -> anyhow::Result<()> {
    BEAM_CLIENT
        .handle_sockets(|task, incoming| async move {
            info!("[Beam] Starting socket polling...");
            // print_file(task, incoming).await;
            if let Err(e) = save_file_as_s3(task, incoming).await {
                tracing::error!("save_file_as_s3 failed: {e:#}");
            }
            Ok(())
        })
        .await?;
    Ok(())
}

async fn save_file_as_s3(
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
    let meta: FileMeta =
        serde_json::from_value(socket_task.metadata).context("Failed to deserialize metadata")?;
    upload_stream_to_s3(&CONFIG.s3_bucket, &CONFIG.s3_key, incoming).await
    // let mut file = tokio::fs::File::create(dir.join(meta.suggested_name.unwrap_or("study_id".to_string()))).await?;
    // tokio::io::copy(&mut incoming, &mut file).await?;
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
