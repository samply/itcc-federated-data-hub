use crate::pseudonym::handler::create_session;
use crate::test::test_config;
use crate::utils::error_type::ErrorType;

#[tokio::test]
async fn test_pseudonym() -> Result<(), ErrorType> {
    let cfg = test_config();
    create_session(&cfg.mainzelliste_url, &cfg.api_mzl_key.as_str()).await
}