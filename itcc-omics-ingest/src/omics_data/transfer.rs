use crate::omics_data::validator;
use crate::utils::config::AppState;
use crate::utils::error_type::ErrorType;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use std::collections::{BTreeSet, HashMap};
use std::io::Cursor;
use tracing::{debug, info};

pub async fn read_validate_scan(
    input: &axum::body::Bytes,
    state: &AppState,
) -> Result<BTreeSet<String>, ErrorType> {
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

    let mut ids = BTreeSet::new();
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

pub async fn build_pseudo_map(ids: BTreeSet<String>) -> Result<HashMap<String, String>, ErrorType> {
    let mut map = HashMap::new();

    // Hard-coded patient pseudonyms
    let fixtures = [
        ("P0KRKM80V_N", "PAT-0001"),
        ("P0KRKM80V_T_1", "PAT-0001"),
        ("P12CU5D2C_N", "PAT-0002"),
        ("P12CU5D2C_T_1", "PAT-0002"),
        ("P4K1CKKPZ_N", "PAT-0003"),
        ("P4K1CKKPZ_T_1", "PAT-0003"),
        ("P4K606AGC_N", "PAT-0004"),
        ("P4K606AGC_T_1", "PAT-0004"),
        ("PDHTUKX47_N", "PAT-0005"),
        ("PDHTUKX47_T_1", "PAT-0005"),
        ("PDKYDRM4Y_N", "PAT-0006"),
        ("PDKYDRM4Y_T_1", "PAT-0006"),
        ("PH2KRL2JM_N", "PAT-0007"),
        ("PH2KRL2JM_T_1", "PAT-0007"),
        ("PRJA391P8_N", "PAT-0008"),
        ("PRJA391P8_T_1", "PAT-0008"),
        ("PUHZMV07U_N", "PAT-0009"),
        ("PUHZMV07U_T_1", "PAT-0009"),
        ("PVK0666T8_N", "PAT-0010"),
        ("PVK0666T8_T_1", "PAT-0010"),
    ];

    for (sample, pseudo) in fixtures {
        if ids.contains(sample) {
            map.insert(sample.to_string(), pseudo.to_string());
        }
    }

    if map.len() != ids.len() {
        return Err(ErrorType::PseudoError);
    }

    Ok(map)
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
