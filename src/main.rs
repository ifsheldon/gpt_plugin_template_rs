use std::sync::Arc;
use axum::http::{header, HeaderValue, Method};
use axum::Router;
use axum::routing::{get, get_service, post};
use accompany::bound;
use clap::Parser;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeFile;
use tower_http::set_header::SetRequestHeaderLayer;
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing::{Level, span, error, info};
use crate::states::light_states;
use crate::utils::{Args, get_or_default_light_states, handle_signal};
use crate::control::{handle_light_color_request, handle_light_control_request};

mod utils;
mod control;
mod states;

const AUTH_STR: &str = "asdfghjkl_light_control";

async fn legal_info() -> &'static str {
    "This is a sample server for the GPT Plugin tutorial"
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();
    let span = span!(Level::INFO, "light_control_server");
    bound!{
        with _guard = span.enter() => {
            info!("Starting light control server on port {}", args.port);
        }
    } // add scope to early drop the guard to prevent the span from being captured by the signal handler

    let shared_light_states = Arc::new(RwLock::new(get_or_default_light_states().await));
    let app = Router::new()
        .route("/lobechat/light_color", post(handle_light_color_request))
        .route("/lobechat/light_control", post(handle_light_control_request))
        .layer(SetRequestHeaderLayer::overriding(header::CONTENT_TYPE, HeaderValue::from_static("application/json"))) // specially for LobeChat due to https://github.com/lobehub/lobe-chat/issues/2216
        .route("/light_color", post(handle_light_color_request))
        .route("/light_control", post(handle_light_control_request))
        .route("/states", get(light_states))
        .with_state(shared_light_states.clone()) // for routes that need shared light states
        .layer(ValidateRequestHeaderLayer::bearer(AUTH_STR)) // for routes that require authorization, simple bearer checking
        // metadata routes
        .route("/legal", get(legal_info))
        .route("/logo.png", get_service(ServeFile::new("src/metadata_files/logo.png")))
        .route("/openapi.json", get_service(ServeFile::new("src/metadata_files/openapi.json")))
        .route("/.well-known/ai-plugin.json", get_service(ServeFile::new("src/metadata_files/ai-plugin.json")))
        // metadata for LobeChat
        .route("/lobechat/openapi.json", get_service(ServeFile::new("src/metadata_files/openapi-lobechat.json")))
        .route("/lobechat/.well-known/ai-plugin.json", get_service(ServeFile::new("src/metadata_files/ai-plugin-lobechat.json")))
        // for all routes, simple defense against DoS attacks
        .layer(ConcurrencyLimitLayer::new(128))
        .layer(RequestBodyLimitLayer::new(1024 * 1024))// 1MB
        // allow requests from any origin
        .layer(CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any))
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(format!("0.0.0.0:{}", args.port)).await.unwrap();
    let server = axum::serve(listener, app);
    let graceful = server.with_graceful_shutdown(handle_signal(shared_light_states, span));
    if let Err(e) = graceful.await {
        error!("Server error: {}", e);
    }
}
