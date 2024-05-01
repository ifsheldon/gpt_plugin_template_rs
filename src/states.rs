use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use crate::utils::SharedLightStates;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightColor {
    Red,
    Green,
    Blue,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightStatus {
    On,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightStates {
    pub color: LightColor,
    pub status: LightStatus,
}

impl Default for LightStates {
    fn default() -> Self {
        LightStates {
            color: LightColor::White,
            status: LightStatus::Off,
        }
    }
}

#[instrument]
pub async fn light_states(State(states): State<SharedLightStates>) -> Json<LightStates> {
    Json(states.read().await.clone())
}
