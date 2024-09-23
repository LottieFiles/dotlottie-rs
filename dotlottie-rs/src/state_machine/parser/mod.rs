use serde::Deserialize;

use crate::errors::StateMachineError;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StringNumberBool {
    String(String),
    F32(f32),
    Bool(bool),
}

#[derive(Clone, Copy, Deserialize, Debug, PartialEq)]
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
    pub initial: String,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum ActionJson {
    OpenUrl {
        url: String,
    },
    ThemeAction {
        theme_id: String,
        target: Option<String>,
    },
    Increment {
        trigger_name: String,
        value: Option<f32>,
    },
    Decrement {
        trigger_name: String,
        value: Option<f32>,
    },
    Toggle {
        trigger_name: String,
    },
    SetBoolean {
        trigger_name: String,
        value: bool,
    },
    SetString {
        trigger_name: String,
        value: String,
    },
    SetNumeric {
        trigger_name: String,
        value: f32,
    },
    Fire {
        trigger_name: String,
    },
    SetExpression {
        layer_name: String,
        property_index: u32,
        var_name: String,
        value: f32,
    },
    SetTheme {
        theme_id: String,
    },
    SetFrame {
        value: f32,
    },
    SetSlot {
        value: String,
    },
    FireCustomEvent {
        value: String,
    },
}

// Type is the actual "type" declared in the state machine State json
// This allows serde to determine which struct to deserialize the json into
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum StateJson {
    PlaybackState {
        name: String,
        transitions: Vec<TransitionJson>,
        animation_id: Option<String>,
        r#loop: Option<bool>,
        autoplay: Option<bool>,
        mode: Option<String>,
        speed: Option<f32>,
        segment: Option<String>,
        background_color: Option<u32>,
        use_frame_interpolation: Option<bool>,
        entry_actions: Option<Vec<ActionJson>>,
        exit_actions: Option<Vec<ActionJson>>,
    },
    GlobalState {
        name: String,
        transitions: Vec<TransitionJson>,
        entry_actions: Option<Vec<ActionJson>>,
        exit_actions: Option<Vec<ActionJson>>,
    },
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum TransitionGuardJson {
    Numeric {
        trigger_name: String,
        condition_type: TransitionGuardConditionType,
        compare_to: StringNumberBool,
    },
    String {
        trigger_name: String,
        condition_type: TransitionGuardConditionType,
        compare_to: StringNumberBool,
    },
    Boolean {
        trigger_name: String,
        condition_type: TransitionGuardConditionType,
        compare_to: StringNumberBool,
    },
    Event {
        trigger_name: String,
    },
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
        to_state: String,
        guards: Option<Vec<TransitionGuardJson>>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ListenerJson {
    PointerUp {
        layer_name: Option<String>,
        actions: Vec<ActionJson>,
    },
    PointerDown {
        layer_name: Option<String>,
        actions: Vec<ActionJson>,
    },
    PointerEnter {
        layer_name: Option<String>,
        actions: Vec<ActionJson>,
    },
    PointerExit {
        layer_name: Option<String>,
        actions: Vec<ActionJson>,
    },
    PointerMove {
        layer_name: Option<String>,
        actions: Vec<ActionJson>,
    },
    OnComplete {
        state_name: Option<String>,
        actions: Vec<ActionJson>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TriggerJson {
    Numeric { name: String, value: f32 },
    String { name: String, value: String },
    Boolean { name: String, value: bool },
    Event { name: String },
}

#[derive(Deserialize, Debug)]
pub struct StateMachineJson {
    pub descriptor: DescriptorJson,
    pub states: Vec<StateJson>,
    pub listeners: Option<Vec<ListenerJson>>,
    pub triggers: Option<Vec<TriggerJson>>,
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
