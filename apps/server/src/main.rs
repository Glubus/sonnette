mod commands;
mod handler;
mod state;
mod sync;

use axum::routing::get;
use axum::Router;
use handler::ws_handler;
use state::AppState;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("server=debug,tower_http=debug")
        .init();

    let state = AppState::new();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
