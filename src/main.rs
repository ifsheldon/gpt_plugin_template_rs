use std::sync::Arc;
use axum::http::Method;
use axum::Router;
use axum::routing::{get, get_service, post};
use clap::Parser;
use log::{error, info};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeFile;
use tower_http::validate_request::ValidateRequestHeaderLayer;
use crate::states::light_states;
use crate::utils::{Args, get_or_default_light_states, handle_signal};
use crate::control::{handle_light_color_request, handle_light_control_request};

mod utils;
mod control;
mod states;

const AUTH_STR: &str = "Bearer asdfghjkl_light_control";

async fn legal_info() -> &'static str {
    "This is a sample server for the GPT Plugin tutorial"
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    env_logger::init();
    info!("{:?}", args);
    // simple authorization
    let validate_bearer = ValidateRequestHeaderLayer::bearer(AUTH_STR);
    // allow requests from any origin
    let allow_cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);
    // simple defense against DoS attacks
    let limit_body = RequestBodyLimitLayer::new(1024 * 1024); // 1MB
    let limit_concurrency = ConcurrencyLimitLayer::new(128);

    let shared_light_states = Arc::new(RwLock::new(get_or_default_light_states().await));
    let app = Router::new()
        .route("/light_color", post(handle_light_color_request))
        .route("/light_control", post(handle_light_control_request))
        .route("/states", get(light_states))
        .with_state(shared_light_states.clone()) // for routes that need shared light states
        .layer(validate_bearer) // for routes that require authorization
        .route("/legal", get(legal_info))
        // TODO: add metadata files
        .route("/logo.png", get_service(ServeFile::new("src/backend/metadata_files/logo.png")))
        .route("/openapi.yaml", get_service(ServeFile::new("src/backend/metadata_files/openapi.yaml")))
        .route("/.well-known/ai-plugin.json", get_service(ServeFile::new("src/backend/metadata_files/ai-plugin.json")))
        // for all routes
        .layer(allow_cors)
        .layer(limit_body)
        .layer(limit_concurrency);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", args.port)).await.unwrap();
    let server = axum::serve(listener, app);
    let graceful = server.with_graceful_shutdown(handle_signal(shared_light_states));
    if let Err(e) = graceful.await {
        error!("Server error: {}", e);
    }
}
