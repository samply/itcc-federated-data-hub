use crate::s3::{get_object, save_files_s3};
use crate::utils::config::FileMeta;
use crate::{BEAM_CLIENT, DATALAKE_CONFIG};
use anyhow::Context;
use beam_lib::SocketTask;
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
    let meta: FileMeta =
        serde_json::from_value(socket_task.metadata).context("Failed to deserialize metadata")?;
    save_files_s3(
        &DATALAKE_CONFIG.s3_bucket,
        incoming,
        &meta.suggested_name.clone().unwrap(),
    )
    .await?;
    get_object(&DATALAKE_CONFIG.s3_bucket, &meta.suggested_name.unwrap()).await?;
    Ok(())
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
