use crate::cbio_portal::data::SampleId;
use anyhow::Context;
use polars::prelude::{
    CsvParseOptions, CsvReadOptions, CsvReader, ParquetCompression, ParquetWriter, SerReader,
};
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;

pub fn maf_to_parquet(
    maf_path: &std::path::Path,
    parquet_path: &std::path::Path,
) -> anyhow::Result<HashSet<SampleId>> {
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

    let sample_ids: HashSet<SampleId> = df
        .column("Tumor_Sample_Barcode")?
        .str()?
        .into_iter()
        .flatten()
        .map(SampleId::new)
        .collect::<anyhow::Result<_>>()?;

    let mut out = File::create(parquet_path)?;
    ParquetWriter::new(&mut out)
        .with_compression(ParquetCompression::Zstd(None))
        .finish(&mut df)?;

    Ok(sample_ids)
}
