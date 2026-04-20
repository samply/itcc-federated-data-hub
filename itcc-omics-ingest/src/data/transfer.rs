use crate::data::validator;
use crate::utils::config::AppState;
use crate::utils::error_type::ErrorType;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use tracing::{debug, info};

/// Parses and validates a tab-separated MAF (Mutation Annotation Format) file,
/// returning a deduplicated set of all sample IDs found in the file.
///
/// Specifically:
/// - Parses the TSV with `#`-prefixed comment lines skipped
/// - Validates the headers against the required omics column schema
/// - Extracts all non-empty values from `Tumor_Sample_Barcode` and
///   `Matched_Norm_Sample_Barcode` columns into a [`HashSet`]
///
/// # Errors
/// Returns [`ErrorType`] if the headers are missing or invalid, a required
/// column is absent, or a CSV parsing error occurs.
pub async fn read_validate_scan(
    input: &axum::body::Bytes,
    state: &AppState,
) -> Result<HashSet<String>, ErrorType> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .comment(Some(b'#'))
        .has_headers(true)
        .flexible(false)
        .from_reader(Cursor::new(&input));

    let headers = match rdr.headers() {
        Ok(h) => h.clone(),
        Err(e) => {
            tracing::error!("Failed to read headers: {e}");
            return Err(ErrorType::MafEmptyHeader);
        }
    };
    validator::schema_validate(&headers, &state.required_omics_columns)?;
    let tumor_idx = headers
        .iter()
        .position(|c| c == "Tumor_Sample_Barcode")
        .ok_or(ErrorType::MafEmptyHeader)?;
    let normal_idx = headers
        .iter()
        .position(|c| c == "Matched_Norm_Sample_Barcode")
        .ok_or(ErrorType::MafEmptyHeader)?;

    debug!("Tumor_Sample_Barcode: {}", tumor_idx);
    debug!("Matched Norm_Sample_Barcode: {}", normal_idx);

    let mut ids = HashSet::new();
    let mut invalid_ids = Vec::new();
    for row in rdr.records() {
        let rec = row.map_err(|_| ErrorType::CsvError)?;

        for idx in [tumor_idx, normal_idx] {
            if let Some(v) = rec.get(idx) {
                let v = v.trim();
                if v.is_empty() {
                    continue;
                }
                if !v.contains('_') {
                    invalid_ids.push(v.to_string());
                } else {
                    ids.insert(v.to_string());
                }
            }
        }
    }

    if !invalid_ids.is_empty() {
        tracing::error!(
            invalid_sample_ids = ?invalid_ids,
            "Sample IDs do not match expected pattern {{base}}_{{suffix}}"
        );
        return Err(ErrorType::MafInvalidSampleId(invalid_ids));
    }
    Ok(ids)
}

/// Rewrites a MAF file's sample ID columns in-place with their pseudonyms.
///
/// Reads the tab-separated input bytes, replaces every value in
/// `Tumor_Sample_Barcode` and `Matched_Norm_Sample_Barcode` with the
/// corresponding pseudonym from `pseudo`, and returns the rewritten file
/// as a byte vector with all other columns and comment lines preserved.
///
/// # Errors
/// Returns [`ErrorType`] if headers are missing, a required column is absent,
/// a sample ID has no entry in `pseudo`, or a CSV read/write error occurs.
pub fn sanitize_maf_bytes(
    input: &axum::body::Bytes,
    pseudo: &HashMap<String, String>,
) -> Result<Vec<u8>, ErrorType> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .comment(Some(b'#'))
        .has_headers(true)
        .flexible(false)
        .from_reader(Cursor::new(input));

    let headers = match rdr.headers() {
        Ok(h) => h.clone(),
        Err(e) => {
            tracing::error!("Failed to read headers: {e}");
            return Err(ErrorType::MafEmptyHeader);
        }
    };
    let tumor_idx = headers
        .iter()
        .position(|c| c == "Tumor_Sample_Barcode")
        .ok_or(ErrorType::MafEmptyHeader)?;
    let normal_idx = headers
        .iter()
        .position(|c| c == "Matched_Norm_Sample_Barcode")
        .ok_or(ErrorType::MafEmptyHeader)?;

    let mut out = Vec::<u8>::with_capacity(input.len());
    let mut wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_writer(&mut out);

    wtr.write_record(&headers)
        .map_err(|_| ErrorType::CsvError)?;

    for row in rdr.byte_records() {
        let rec: ByteRecord = row.map_err(|_| ErrorType::CsvError)?;
        let mut out_rec = ByteRecord::new();

        for (i, field) in rec.iter().enumerate() {
            if i == tumor_idx || i == normal_idx {
                let s = std::str::from_utf8(field)
                    .map_err(|_| ErrorType::CsvError)?
                    .trim();
                if s.is_empty() {
                    out_rec.push_field(field);
                } else {
                    let p = pseudo.get(s).ok_or_else(|| ErrorType::CsvError)?;
                    out_rec.push_field(p.as_bytes());
                }
            } else {
                out_rec.push_field(field);
            }
        }

        wtr.write_byte_record(&out_rec)
            .map_err(|_| ErrorType::MafWriteError)?;
    }

    wtr.flush().map_err(|_| ErrorType::MafWriteError)?;
    drop(wtr);
    info!("Matched params and pseudomisation completed");
    Ok(out)
}
