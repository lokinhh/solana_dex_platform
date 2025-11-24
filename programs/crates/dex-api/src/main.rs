mod routes;
mod state;

use std::net::SocketAddr;

use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("dex_api=info".parse()?))
        .init();

    let host = std::env::var("RUST_API_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("RUST_API_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8092);

    let state = AppState::new();
    let app = Router::new()
        .merge(routes::router())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!(%addr, "dex_api_listening");
    axum::serve(listener, app).await?;
    Ok(())
}
