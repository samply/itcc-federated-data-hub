use anyhow::Context;
use beam_lib::SocketTask;
use itcc_omics_lib::beam::Ack;
use itcc_omics_lib::error_type::LibError;
use itcc_omics_lib::fhir::blaze::post_patient_fhir_bundle;
use itcc_omics_lib::fhir::bundle::Bundle;
use polars::prelude::{
    CsvParseOptions, CsvReadOptions, CsvReader, ParquetCompression, ParquetWriter, SerReader,
};
use reqwest::Url;
use std::fs::File;
use std::io::{BufReader, Write};
use tempfile::NamedTempFile;
use tokio::io::AsyncRead;
use tracing::{debug, info};

pub fn maf_to_parquet(
    maf_path: &std::path::Path,
    parquet_path: &std::path::Path,
) -> anyhow::Result<()> {
    let file = File::open(maf_path)?;
    let reader = BufReader::new(file);

    let mut df = CsvReader::new(reader)
        .with_options(
            CsvReadOptions::default()
                .with_has_header(true)
                .with_parse_options(
                    CsvParseOptions::default()
                        .with_separator(b'\t')
                        .with_comment_prefix(Some("#")),
                ),
        )
        .finish()
        .context("Failed to read MAF/TSV")?;

    let mut out = File::create(parquet_path)?;
    ParquetWriter::new(&mut out)
        .with_compression(ParquetCompression::Zstd(None))
        .finish(&mut df)?;

    Ok(())
}

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

pub fn decompress_zstd_to_tempfile(
    zst_path: &std::path::Path,
) -> anyhow::Result<std::path::PathBuf> {
    let input = std::fs::File::open(zst_path)?;
    let mut decoder = zstd::stream::read::Decoder::new(input)?;

    let tmp = NamedTempFile::new()?;
    let out_path = tmp.path().to_path_buf();
    let (mut out_file, out_path) = tmp.keep()?;

    std::io::copy(&mut decoder, &mut out_file)?;
    out_file.flush()?;

    Ok(out_path)
}
