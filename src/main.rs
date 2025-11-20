mod routes;

use crate::routes::build_router;

use std::{env, fs, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Clone)]
pub struct AppState {
    pub upload_dir: Arc<PathBuf>,
}

#[tokio::main]
async fn main() {
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("omics_endpoint=info".parse().unwrap()),
        )
        .init();

    let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
    let upload_dir_path = PathBuf::from(&upload_dir);

    fs::create_dir_all(&upload_dir_path).unwrap_or_else(|e| {
        panic!(
            "Failed to create upload dir {}: {e}",
            upload_dir_path.display()
        )
    });

    let state = AppState {
        upload_dir: Arc::new(upload_dir_path),
    };

    let app = build_router(state);

    info!("Starting server token ON!");
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    info!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
