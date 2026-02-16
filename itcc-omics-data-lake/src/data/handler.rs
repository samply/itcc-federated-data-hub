use anyhow::Context;
use polars::prelude::{
    CsvParseOptions, CsvReadOptions, CsvReader, ParquetCompression, ParquetWriter, SerReader,
};
use std::fs::File;
use std::io::{BufReader, Write};
use tempfile::NamedTempFile;

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
