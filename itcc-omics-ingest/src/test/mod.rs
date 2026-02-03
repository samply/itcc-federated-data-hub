mod transfer;

use crate::utils::config::{AppState, Config};
use beam_lib::reqwest::Url;
use beam_lib::AppId;

fn test_config() -> Config {
    Config {
        api_key: "omics".to_string(),
        beam_url: Url::parse("http://beam-proxy:8081").unwrap(),
        partner_id: "itcc-inform".to_string(),
        blaze_url: Url::parse("http://host.docker.internal:8081/fhir").unwrap(),
        mainzelliste_url: Url::parse("http://host.docker.internal:7878").unwrap(),
        beam_secret: "App1Secret".to_string(),
        beam_id: AppId::new_unchecked("app1.proxy1.broker"),
        data_lake_id: AppId::new_unchecked("app1.proxy2.broker"),
        zstd_level: 3,
        required_omics_columns: vec![
            "Hugo_Symbol".to_string(),
            "Chromosome".to_string(),
            "Start_Position".to_string(),
            "End_Position".to_string(),
        ],
    }
}

#[test]
fn app_state_is_derived_from_config() {
    let cfg = test_config();
    let state = AppState::from(&cfg);

    assert_eq!(state.api_key, "omics");
    assert_eq!(state.zstd_level, 3);
    assert_eq!(state.data_lake_id, cfg.data_lake_id);
    assert_eq!(state.required_omics_columns, cfg.required_omics_columns);
}
