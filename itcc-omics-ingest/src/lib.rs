pub mod beam;
mod controllers;
pub mod data;
pub mod pseudonym;
#[cfg(test)]
pub mod test;
pub mod utils;

use crate::controllers::extractors::api_key_check;
use crate::controllers::{health, omics, patient};
use crate::utils::config::AppState;
use crate::utils::config::IngestConfig;
use axum::middleware::from_fn_with_state;
use axum::Router;
use clap::Parser;
use once_cell::sync::Lazy;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

pub static CONFIG_INGEST: Lazy<IngestConfig> = Lazy::new(<IngestConfig as clap::Parser>::parse);
pub async fn run_with_config() {
    let state = Arc::new(AppState::from(&*CONFIG_INGEST));

    let app = create_router(state);

    info!("Starting server token ON!");
    let addr: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 6080));
    info!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.expect("Can't listen to port");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("Can't start server");
}

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .merge(omics::routers())
        .merge(patient::routers())
        .route_layer(from_fn_with_state(app_state.clone(), api_key_check))
        .merge(health::routers())
        .with_state(app_state)
}

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("itcc_omics_ingest=info,tower_http=debug,info")),
        )
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .pretty(),
            // .json()
        )
        .init();
}
