use accompany::bound;
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::{instrument, info};
use crate::states::{LightColor, LightStatus};
use crate::utils::SharedLightStates;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ColorAction {
    ToRed,
    ToGreen,
    ToBlue,
    ToWhite,
    Reset,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LightColorRequest {
    pub action: ColorAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LightAction {
    TurnOn,
    TurnOff,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LightControlRequest {
    pub action: LightAction,
}


#[instrument]
pub async fn handle_light_color_request(State(states): State<SharedLightStates>,
                                        Json(request): Json<LightColorRequest>) -> String {
    info!("Light color request: {:?}", request);
    let mut state = states.write().await;
    match request.action {
        ColorAction::ToRed => {
            state.color = LightColor::Red;
            "Light color set to red".to_string()
        }
        ColorAction::ToGreen => {
            state.color = LightColor::Green;
            "Light color set to green".to_string()
        }
        ColorAction::ToBlue => {
            state.color = LightColor::Blue;
            "Light color set to blue".to_string()
        }
        ColorAction::ToWhite => {
            state.color = LightColor::White;
            "Light color set to white".to_string()
        }
        ColorAction::Reset => {
            state.color = LightColor::White;
            "Light color reset to white".to_string()
        }
    }
}

#[instrument]
pub async fn handle_light_control_request(State(states): State<SharedLightStates>,
                                          Json(request): Json<LightControlRequest>) -> String {
    info!("Light control request: {:?}", request);
    let status = bound! {
        with state_guard = states.read().await => {
            state_guard.status
        }
    };  // add scope to early release the lock to avoid deadlock later
    match (request.action, status) {
        (LightAction::TurnOn, LightStatus::On) => "The light is already on".to_string(),
        (LightAction::TurnOff, LightStatus::Off) => "The light is already off".to_string(),
        (LightAction::TurnOn, LightStatus::Off) => {
            let mut state = states.write().await;
            state.status = LightStatus::On;
            "Light turned on".to_string()
        }
        (LightAction::TurnOff, LightStatus::On) => {
            let mut state = states.write().await;
            state.status = LightStatus::Off;
            "Light turned off".to_string()
        }
    }
}