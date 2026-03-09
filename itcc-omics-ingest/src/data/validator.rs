use crate::utils::error_type::ErrorType;
use csv::StringRecord;
use std::collections::{HashMap, HashSet};
use tracing::debug;

pub struct HeaderInfo {
    pub index: HashMap<String, usize>,
    pub names: Vec<String>,
}

pub fn schema_validate(header: &StringRecord, required: &Vec<String>) -> Result<(), ErrorType> {
    let header_info = build_header_info(header)?;
    validate_required_columns(&header_info, required)
}

fn build_header_info(header: &StringRecord) -> Result<HeaderInfo, ErrorType> {
    let mut seen = HashSet::new();
    let mut index = HashMap::new();
    let mut names = vec![];

    for (i, h) in header.iter().enumerate() {
        //debug!("index: {}", i);
        //debug!("name: {}", h);
        let name = h.trim().to_string();
        if name.is_empty() {
            debug!("Header {} is empty", i);
            Err(ErrorType::MafEmptyHeader)?
        }
        if !seen.insert(name.clone()) {
            debug!("duplicate header name: {}", name);
            Err(ErrorType::MafDuplicateHeader)?
        }
        index.insert(name.clone(), i);
        names.push(name)
    }
    Ok(HeaderInfo { index, names })
}

fn validate_required_columns(h: &HeaderInfo, required: &Vec<String>) -> Result<(), ErrorType> {
    for r in required {
        if !h.index.contains_key(r) {
            debug!("index does not exist {}", r);
            debug!("index contains {}", r);
            Err(ErrorType::MafMissingColumn)?
        }
    }
    Ok(())
}
