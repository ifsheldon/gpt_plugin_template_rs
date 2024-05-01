use std::sync::Arc;
use clap::Parser;
use tracing::{warn, Span, info};
use tokio::sync::RwLock;
use crate::states::LightStates;

const LIGHT_STATES_FILE_PATH: &str = "./storage/light_states.json";

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(long, default_value_t = 12345)]
    pub port: u16,
}

pub type SharedLightStates = Arc<RwLock<LightStates>>;

pub async fn get_or_default_light_states() -> LightStates {
    match tokio::fs::read_to_string(LIGHT_STATES_FILE_PATH).await {
        Ok(json) => match serde_json::from_str(&json) {
            Ok(states) => return states,
            Err(e) => warn!("Failed to parse light states from file: {}", e)
        }
        Err(e) => warn!("Failed to read light states from file: {}", e)
    }
    warn!("Using default light states");
    return LightStates::default();
}

pub async fn handle_signal(shared_light_states: SharedLightStates, span: Span) {
    let _guard = span.enter();
    if let Err(e) = tokio::signal::ctrl_c().await {
        warn!(parent: &span, "Failed to listen for shutdown signal: {}", e);
    } else {
        info!(parent: &span, "Received shutdown signal");
        let states = shared_light_states.read().await;
        let json = serde_json::to_string(&*states).unwrap();
        match tokio::fs::write(LIGHT_STATES_FILE_PATH, json).await {
            Ok(_) => info!(parent: &span, "Saved light states to file"),
            Err(e) => warn!(parent: &span, "Failed to save light states to file: {}", e),
        }
    }
}