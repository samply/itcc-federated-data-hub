use anyhow::Context;
use beam_lib::SocketTask;
use itcc_omics_lib::beam::Ack;
use itcc_omics_lib::cbio_portal::data::SampleId;
use itcc_omics_lib::fhir::blaze::post_patient_fhir_bundle;
use itcc_omics_lib::fhir::bundle::Bundle;
use reqwest::Url;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use tempfile::NamedTempFile;
use tokio::io::AsyncRead;
use tracing::{debug, info};

pub async fn handle_fhir_bundle(client: &reqwest::Client, blaze_url: &Url, bundle: Bundle) -> Ack {
    // store to blaze
    debug!("Received Bundle Task");
    match post_patient_fhir_bundle(client, blaze_url, &bundle).await {
        Ok(_) => Ack {
            ok: true,
            message: None,
        },
        Err(_) => Ack {
            ok: false,
            message: None,
        },
    }
}

pub async fn print_file(
    socket_task: SocketTask,
    mut incoming: impl AsyncRead + Unpin,
) -> anyhow::Result<()> {
    info!("Incoming file from {}", socket_task.from);
    tokio::io::copy(&mut incoming, &mut tokio::io::stdout()).await?;
    info!("Done printing file from {}", socket_task.from);
    Ok(())
}
