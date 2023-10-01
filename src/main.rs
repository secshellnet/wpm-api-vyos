pub mod views;
pub mod schemas;
mod cfg;
mod utils;

use clap::Parser;
use tracing::info;
use axum::{
    Router,
    routing::{get,post,delete},
};
use tower_http::validate_request::ValidateRequestHeaderLayer;
use std::sync::Arc;

use crate::cfg::ClientCfg;
use crate::views::{
    wpm_redirect,
    get_peer,
    add_peer,
    delete_peer,
};
use crate::schemas::ConfigState;

#[tokio::main]
async fn main() {
    let config = Arc::new(ClientCfg::parse());

    tracing_subscriber::fmt::init();

    // TODO think about enabling tls support using rustls
    info!("Listening on {}://{}...", "http", config.listen_addr);

    let api_router = Router::new()
        .route("/peer/:identifier", get(get_peer))
        .route("/peer/", post(add_peer))
        .route("/peer/:identifier", delete(delete_peer))
        .layer(ValidateRequestHeaderLayer::bearer(config.secret.as_str()));

    let app = Router::new()
        .route("/", get(wpm_redirect))
        .nest("/api", api_router)
        .with_state(ConfigState{
            config: config.clone(),
        });

    axum::Server::bind(&config.listen_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
