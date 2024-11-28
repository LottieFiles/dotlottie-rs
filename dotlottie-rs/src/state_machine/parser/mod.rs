use serde::Deserialize;

use crate::state_machine::errors::StateMachineError;

#[derive(Deserialize, Debug, PartialEq)]
pub enum StateActionType {
    URLAction,
    ThemeAction,
    SoundAction,
    LogAction,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StringNumberBool {
    String(String),
    F32(f32),
    Bool(bool),
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum TransitionGuardType {
    Numeric,
    String,
    Boolean,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum ContextJsonType {
    Numeric,
    String,
    Boolean,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum TransitionJsonType {
    Transition,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum ListenerJsonType {
    PointerUp,
    PointerDown,
    PointerEnter,
    PointerExit,
    PointerMove,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum TransitionGuardConditionType {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Deserialize, Debug)]
pub struct DescriptorJson {
    pub id: String,
    pub initial: u32,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum StateActionJson {
    URLAction {
        url: String,
        target: Option<String>,
    },
    ThemeAction {
        theme_id: String,
        target: Option<String>,
    },
    SoundAction {
        sound_id: String,
        target: Option<String>,
    },
    LogAction {
        message: String,
    },
}

// Type is the actual "type" declared in the state machine State json
// This allows serde to determine which struct to deserialize the json into
#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum StateJson {
    PlaybackState {
        name: String,
        animation_id: Option<String>,
        r#loop: Option<bool>,
        autoplay: Option<bool>,
        mode: Option<String>,
        speed: Option<f32>,
        marker: Option<String>,
        segment: Option<Vec<f32>>,
        background_color: Option<u32>,
        use_frame_interpolation: Option<bool>,
        entry_actions: Option<Vec<StateActionJson>>,
        exit_actions: Option<Vec<StateActionJson>>,
        reset_context: Option<String>,
    },
    SyncState {
        name: String,
        frame_context_key: String,
        animation_id: Option<String>,
        background_color: Option<u32>,
        segment: Option<Vec<f32>>,
        entry_actions: Option<Vec<StateActionJson>>,
        exit_actions: Option<Vec<StateActionJson>>,
        reset_context: Option<String>,
    },
    GlobalState {
        name: String,
        entry_actions: Option<Vec<StateActionJson>>,
        exit_actions: Option<Vec<StateActionJson>>,
        reset_context: Option<String>,
    },
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct TransitionGuardJson {
    pub r#type: TransitionGuardType,
    pub context_key: String,
    pub condition_type: TransitionGuardConditionType,
    pub compare_to: StringNumberBool,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct NumericEventJson {
    pub value: f32,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct StringEventJson {
    pub value: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct BooleanEventJson {
    pub value: bool,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct OnCompleteEventJson {}

#[derive(Deserialize, Debug, PartialEq)]
pub struct OnPointerDownEventJson {
    pub target: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct OnPointerUpEventJson {
    pub target: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct OnPointerEnterEventJson {
    pub target: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct OnPointerExitEventJson {
    pub target: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct OnPointerMoveEventJson {
    pub target: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TransitionJson {
    Transition {
        from_state: u32,
        to_state: u32,
        guards: Option<Vec<TransitionGuardJson>>,
        numeric_event: Option<NumericEventJson>,
        string_event: Option<StringEventJson>,
        boolean_event: Option<BooleanEventJson>,
        on_complete_event: Option<OnCompleteEventJson>,
        on_pointer_down_event: Option<OnPointerDownEventJson>,
        on_pointer_up_event: Option<OnPointerUpEventJson>,
        on_pointer_enter_event: Option<OnPointerEnterEventJson>,
        on_pointer_exit_event: Option<OnPointerExitEventJson>,
        on_pointer_move_event: Option<OnPointerMoveEventJson>,
    },
}

#[derive(Deserialize, Debug)]
pub struct ListenerJson {
    pub r#type: ListenerJsonType,
    pub target: Option<String>,
    pub action: Option<String>,
    pub value: Option<StringNumberBool>,
    pub context_key: Option<String>,
}

// todo move to enum and add #[serde(tag = "type")]
#[derive(Deserialize, Debug)]
pub struct ContextJson {
    pub r#type: ContextJsonType,
    pub key: String,
    pub value: StringNumberBool,
}

#[derive(Deserialize, Debug)]
pub struct StateMachineJson {
    pub descriptor: DescriptorJson,
    pub states: Vec<StateJson>,
    pub transitions: Vec<TransitionJson>,
    pub listeners: Vec<ListenerJson>,
    pub context_variables: Vec<ContextJson>,
}

pub fn state_machine_parse(json: &str) -> Result<StateMachineJson, StateMachineError> {
    let result: Result<StateMachineJson, _> = serde_json::from_str(json);

    match result {
        Ok(k) => Ok(k),
        Err(err) => Err(StateMachineError::ParsingError {
            reason: err.to_string(),
        }),
    }
}
