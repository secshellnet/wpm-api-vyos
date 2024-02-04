use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};
use clap::Parser;
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::info;

use crate::cfg::ClientCfg;
use crate::schemas::ConfigState;
use crate::views::{get_peers, get_peer, add_peer, delete_peer};

mod cfg;
pub mod schemas;
mod utils;
pub mod views;

#[tokio::main]
async fn main() {
    let config = Arc::new(ClientCfg::parse());

    tracing_subscriber::fmt::init();

    info!("Listening on {}://{}...", "http", config.listen_addr);

    let api_router = Router::new()
        .route("/peer/", get(get_peers))
        .route("/peer/:identifier", get(get_peer))
        .route("/peer/", post(add_peer))
        .route("/peer/:identifier", delete(delete_peer))
        .layer(ValidateRequestHeaderLayer::bearer(config.secret.as_str()));

    let app = Router::new()
        .nest("/api", api_router)
        .with_state(ConfigState {
            config: config.clone(),
        });

    axum::Server::bind(&config.listen_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
