use crate::utils::error_type::ErrorType;
use csv::StringRecord;
use std::collections::{HashMap, HashSet};

pub struct HeaderInfo {
    pub index: HashMap<String, usize>,
    pub names: Vec<String>,
}

pub struct Schema {}

impl Schema {
    pub fn validate(header: &StringRecord, required: &Vec<String>) -> Result<(), ErrorType> {
        let header_info = Self::build_header_info(header)?;
        Self::validate_required_columns(&header_info, required)
    }

    fn build_header_info(header: &StringRecord) -> Result<HeaderInfo, ErrorType> {
        let mut seen = HashSet::new();
        let mut index = HashMap::new();
        let mut names = vec![];

        for (i, h) in header.iter().enumerate() {
            let name = h.trim().to_string();
            if name.is_empty() {
                Err(ErrorType::MafEmptyHeader)?
            }
            if !seen.insert(name.clone()) {
                Err(ErrorType::MafDuplicateHeader)?
            }
        }
        Ok(HeaderInfo { index, names })
    }

    fn validate_required_columns(h: &HeaderInfo, required: &Vec<String>) -> Result<(), ErrorType> {
        for r in required {
            if !h.index.contains_key(r) {
                Err(ErrorType::MafMissingColumn)?
            }
        }
        Ok(())
    }
}
