use serde::Deserialize;

use crate::DotLottieError;

#[derive(Deserialize, Debug)]
pub struct DescriptorJson {
    pub id: String,
    pub initial: u32,
}

#[derive(Deserialize, Debug)]
pub struct StateActionJson {
    pub r#type: String,
    pub url: Option<String>,
    pub target: Option<String>,
    pub theme_id: Option<String>,
    pub sound_id: Option<String>,
    pub message: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct StateJson {
    pub r#type: String,
    pub animation_id: Option<String>,
    pub r#loop: Option<bool>,
    pub autoplay: Option<bool>,
    pub mode: Option<String>,
    pub speed: Option<f32>,
    pub marker: Option<String>,
    pub segment: Option<Vec<f32>>,
    pub background_color: Option<u32>,
    pub use_frame_interpolation: Option<bool>,
    pub entry_actions: Option<Vec<StateActionJson>>,
    pub exit_actions: Option<Vec<StateActionJson>>,
    pub reset_context: Option<bool>,
    pub frame_context_key: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TransitionGuardJson {}

#[derive(Deserialize, Debug)]
pub struct NumericEventJson {
    pub value: f32,
}

#[derive(Deserialize, Debug)]
pub struct StringEventJson {
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct BooleanEventJson {
    pub value: bool,
}

#[derive(Deserialize, Debug)]
pub struct OnCompleteEventJson {}

#[derive(Deserialize, Debug)]
pub struct OnPointerDownEventJson {
    pub target: String,
}

#[derive(Deserialize, Debug)]
pub struct OnPointerUpEventJson {
    pub target: String,
}

#[derive(Deserialize, Debug)]
pub struct OnPointerEnterEventJson {
    pub target: String,
}

#[derive(Deserialize, Debug)]
pub struct OnPointerExitEventJson {
    pub target: String,
}

#[derive(Deserialize, Debug)]
pub struct OnPointerMoveEventJson {
    pub target: String,
}

#[derive(Deserialize, Debug)]
pub struct TransitionJson {
    pub r#type: String,
    pub from_state: u32,
    pub to_state: u32,
    pub guard: Option<Vec<TransitionGuardJson>>,
    pub numeric_event: Option<NumericEventJson>,
    pub string_event: Option<StringEventJson>,
    pub boolean_event: Option<BooleanEventJson>,
    pub on_complete_event: Option<OnCompleteEventJson>,
    pub on_pointer_down_event: Option<OnPointerDownEventJson>,
    pub on_pointer_up_event: Option<OnPointerUpEventJson>,
    pub on_pointer_enter_event: Option<OnPointerEnterEventJson>,
    pub on_pointer_exit_event: Option<OnPointerExitEventJson>,
    pub on_pointer_move_event: Option<OnPointerMoveEventJson>,
}

#[derive(Deserialize, Debug)]
pub struct ListenerJson {
    pub r#type: String,
    pub target: Option<String>,
    pub action: Option<String>,
    pub value: Option<String>,
    pub context_key: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ContextJson {
    pub r#type: String,
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct StateMachineJson {
    pub descriptor: DescriptorJson,
    pub states: Vec<StateJson>,
    pub transitions: Vec<TransitionJson>,
    pub listeners: Vec<ListenerJson>,
    pub context_variables: Vec<ContextJson>,
}

pub fn state_machine_parse(json: &str) -> Result<StateMachineJson, DotLottieError> {
    let result: Result<StateMachineJson, serde_json::Error> = serde_json::from_str(json);

    match result {
        Ok(state_machine_json) => {
            return Ok(state_machine_json);
        }
        Err(err) => Err(DotLottieError::StateMachineError {
            reason: format!("Error parsing state machine definition file: {}", err),
        }),
    }
}
