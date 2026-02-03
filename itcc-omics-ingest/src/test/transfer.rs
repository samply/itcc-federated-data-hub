use crate::utils::error_type::ErrorType;
use std::collections::{BTreeSet, HashMap};

/*
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

 */
