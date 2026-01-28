pub mod beam;
mod controllers;
mod fhir;
pub mod omics_data;
mod pseudonym;
#[cfg(test)]
mod test;
pub mod utils;

use crate::controllers::extractors::api_key_check;
use crate::controllers::{health, omics};
use crate::utils::config::{AppState, Config};
use axum::middleware::from_fn_with_state;
use axum::Router;
use beam_lib::{AppId, BeamClient};
use clap::Parser;
use once_cell::sync::Lazy;
use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;

pub static CONFIG: Lazy<Config> = Lazy::new(Config::parse);
pub static BEAM_CLIENT: Lazy<BeamClient> = Lazy::new(|| {
    BeamClient::new(
        &CONFIG.beam_id,
        &CONFIG.beam_secret,
        CONFIG.beam_url.clone(),
    )
});

pub async fn run_with_config() {
    let state = AppState {
        api_key: CONFIG.api_key.clone(),
        zstd_level: CONFIG.zstd_level,
        required_omics_columns: CONFIG.required_omics_columns.clone(),
        data_lake_id: CONFIG.data_lake_id.clone(),
    };

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
