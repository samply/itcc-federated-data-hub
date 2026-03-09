use crate::data::validator;
use crate::utils::config::AppState;
use crate::utils::error_type::ErrorType;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use tracing::{debug, info};

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
    for row in rdr.records() {
        let rec = row.map_err(|_| ErrorType::CsvError)?;
        if let Some(v) = rec.get(tumor_idx) {
            let v = v.trim();
            if !v.is_empty() {
                ids.insert(v.to_string());
            }
        }
        if let Some(v) = rec.get(normal_idx) {
            let v = v.trim();
            if !v.is_empty() {
                ids.insert(v.to_string());
            }
        }
    }
    Ok(ids)
}

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

pub fn filter_patient_id(ids: &HashSet<String>) -> HashSet<String> {
    ids.into_iter().map(|id| split_base(id)).collect()
}

pub fn split_base(sample: &str) -> String {
    sample
        .split_once("_")
        .map(|x| x.0)
        .unwrap_or(sample)
        .to_string()
}

pub fn insert_base(sample: &str, crypto_id: &str) -> String {
    sample
        .split_once("_")
        .map(|x| format!("{}_{}", crypto_id, x.1))
        .unwrap_or(crypto_id.to_string())
        .to_string()
}
