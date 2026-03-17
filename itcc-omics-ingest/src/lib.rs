pub mod beam;
mod controllers;
mod fhir;
pub mod omics_data;
pub mod pseudonym;
#[cfg(test)]
pub mod test;
pub mod utils;

use crate::controllers::extractors::api_key_check;
use crate::controllers::{health, omics};
use crate::utils::config::Config as IngestConfig;
use crate::utils::config::{AppState, Services};
use axum::middleware::from_fn_with_state;
use axum::Router;
use beam_lib::BeamClient;
use clap::Parser;
use once_cell::sync::Lazy;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;

pub static CONFIG_INGEST: Lazy<IngestConfig> = Lazy::new(<IngestConfig as clap::Parser>::parse);
pub static BEAM_CLIENT: Lazy<BeamClient> = Lazy::new(|| {
    BeamClient::new(
        &CONFIG_INGEST.beam_id,
        &CONFIG_INGEST.beam_secret,
        CONFIG_INGEST.beam_url.clone(),
    )
});
pub async fn run_with_config() {
    let state = AppState::from(&*CONFIG_INGEST);

    let app = create_router(state);

    info!("Starting server token ON!");
    let addr: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 6080));
    info!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.expect("Can't listen to port");
    axum::serve(listener, app.await.into_make_service())
        .await
        .expect("Can't start server");
}

pub async fn create_router(app_state: AppState) -> Router {
    Router::new()
        .merge(omics::routers())
        .route_layer(from_fn_with_state(app_state.clone(), api_key_check))
        .merge(health::routers())
        .with_state(app_state)
}
