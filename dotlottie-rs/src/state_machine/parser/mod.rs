use serde::Deserialize;

use crate::errors::StateMachineError;

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

#[derive(Deserialize, Debug, PartialEq)]
pub enum StateType {
    PlaybackState,
    FinalState,
    SyncState,
    GlobalState,
}

#[derive(Deserialize, Debug)]
pub struct DescriptorJson {
    pub id: String,
    pub initial: u32,
}

// #[derive(Deserialize, Debug, PartialEq)]
// pub struct StateActionJson {
//     pub r#type: StateActionType,
//     pub url: Option<String>,
//     pub target: Option<String>,
//     pub theme_id: Option<String>,
//     pub sound_id: Option<String>,
//     pub message: Option<String>,
// }

// r#type: StateActionType,
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
#[serde(untagged)]
pub enum StateJson {
    PlaybackState(PlaybackState),
    // PlaybackState {
    //     name: String,
    //     // r#type: StateType,
    //     animation_id: Option<String>,
    //     r#loop: Option<bool>,
    //     autoplay: Option<bool>,
    //     mode: Option<String>,
    //     speed: Option<f32>,
    //     marker: Option<String>,
    //     segment: Option<Vec<f32>>,
    //     background_color: Option<u32>,
    //     use_frame_interpolation: Option<bool>,
    //     entry_actions: Option<Vec<StateActionJson>>,
    //     exit_actions: Option<Vec<StateActionJson>>,
    //     reset_context: Option<String>,
    // },
    SyncState {
        name: String,
        // pub r#type: StateType,
        animation_id: Option<String>,
        background_color: Option<u32>,
        use_frame_interpolation: Option<bool>,
        entry_actions: Option<Vec<StateActionJson>>,
        exit_actions: Option<Vec<StateActionJson>>,
        reset_context: Option<String>,
        frame_context_key: String,
    },
    // PlaybackState(PlaybackState),
    // SyncState(SyncState),
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PlaybackState {
    pub name: String,
    pub r#type: StateType,
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
    pub reset_context: Option<String>,
    // pub frame_context_key: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SyncState {
    pub name: String,
    pub r#type: StateType,
    pub animation_id: Option<String>,
    pub background_color: Option<u32>,
    pub use_frame_interpolation: Option<bool>,
    pub entry_actions: Option<Vec<StateActionJson>>,
    pub exit_actions: Option<Vec<StateActionJson>>,
    pub reset_context: Option<String>,
    pub frame_context_key: Option<String>,
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

#[derive(Deserialize, Debug, PartialEq)]
pub struct TransitionJson {
    pub r#type: TransitionJsonType,
    pub from_state: u32,
    pub to_state: u32,
    pub guards: Option<Vec<TransitionGuardJson>>,
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
    pub r#type: ListenerJsonType,
    pub target: Option<String>,
    pub action: Option<String>,
    pub value: Option<StringNumberBool>,
    pub context_key: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ContextJson {
    pub r#type: ContextJsonType,
    pub key: String,
    pub value: StringNumberBool,
}

impl ContextJson {
    pub fn new() -> ContextJson {
        Self {
            r#type: ContextJsonType::String,
            key: String::new(),
            value: StringNumberBool::String(String::new()),
        }
    }
}

impl Default for ContextJson {
    fn default() -> Self {
        Self::new()
    }
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
    let r: Result<StateMachineJson, _> = serde_json::from_str(json);

    if r.is_err() {
        println!(
            "Error parsing state machine definition file: {}",
            r.err().unwrap()
        );
    }

    // Some Deserializer.
    let jd = &mut serde_json::Deserializer::from_str(json);

    let result: Result<StateMachineJson, _> = serde_path_to_error::deserialize(jd);

    match result {
        Ok(k) => serde_json::from_str(json).map_err(|err| {
            println!("{}", err);

            StateMachineError::ParsingError {
                reason: format!("Error parsing state machine definition file: {}", err),
            }
        }),
        Err(err) => {
            let path = err.path().to_string();
            println!("Error path -> {}", path);

            Err(StateMachineError::ParsingError {
                reason: "failed".to_owned(),
            })
            // assert_eq!(path, "dependencies.serde.version");
        }
    }
}
