mod controllers;
mod config;

use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};
use axum::Router;
use tokio::net::TcpListener;
use tracing::info;
use once_cell::sync::Lazy;
use clap::Parser;
use crate::config::Config;
use crate::controllers::{health, omics};

pub static CONFIG: Lazy<Config> = Lazy::new(Config::parse);

#[derive(Clone)]
pub struct AppState {
    pub upload_dir: Arc<PathBuf>,
}

pub async fn run_with_config() -> anyhow::Result<()> {
    let upload_dir = PathBuf::from(&CONFIG.upload_dir);

    fs::create_dir_all(&upload_dir).unwrap_or_else(|e| {
        panic!(
            "Failed to create upload dir {}: {e}",
            upload_dir.display()
        )
    });

    let state = AppState {
        upload_dir: Arc::new(upload_dir),
    };

    let app = create_router(state);

    info!("Starting server token ON!");
    let addr: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 8040));
    info!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.await.into_make_service()).await?;
    Ok(())
}

pub async fn create_router(app_state: AppState) -> Router {
    Router::new()
        .merge(omics::routers())
        .merge(health::routers())
        .with_state(app_state)
}
