use crate::pseudonym::handler::create_session;
use crate::test::{test_app_state, test_config};
use crate::utils::error_type::ErrorType;

#[ignore = "Require mainzelliste"]
#[tokio::test]
async fn test_pseudonym() -> Result<(), ErrorType> {
    let app_state = test_app_state();
    create_session(&app_state).await
}
