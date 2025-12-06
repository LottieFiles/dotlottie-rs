use serde::Deserialize;
use std::{rc::Rc, sync::RwLock};

use crate::{
    inputs::InputTrait, state_machine::StringBool, DotLottiePlayerContainer, Event, GradientStop,
    ImageValue,
};

use super::{state_machine::StringNumber, StateMachineEngine};

pub mod open_url_policy;
pub mod whitelist;

#[derive(Debug)]
pub enum StateMachineActionError {
    ExecuteError,
    ParsingError,
}

pub trait ActionTrait {
    fn execute(
        &self,
        engine: &mut StateMachineEngine,
        player: Rc<RwLock<DotLottiePlayerContainer>>,
        run_pipeline: bool,
        called_from_interaction: bool,
    ) -> Result<(), StateMachineActionError>;
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "type")]
pub enum Action {
    OpenUrl {
        url: String,
        target: String,
    },
    Increment {
        input_name: String,
        value: Option<StringNumber>,
    },
    Decrement {
        input_name: String,
        value: Option<StringNumber>,
    },
    Toggle {
        input_name: String,
    },
    SetBoolean {
        input_name: String,
        value: StringBool,
    },
    SetString {
        input_name: String,
        value: String,
    },
    SetNumeric {
        input_name: String,
        value: StringNumber,
    },
    SetGlobalColor {
        global_input_id: String,
        value: Vec<StringNumber>,
    },
    SetGlobalGradient {
        global_input_id: String,
        value: Vec<GradientStop>,
    },
    // SetGlobalImage {
    //     global_input_id: String,
    //     value: ImageValue,
    // },
    SetGlobalString {
        global_input_id: String,
        value: String,
    },
    SetGlobalNumeric {
        global_input_id: String,
        value: f32,
    },
    SetGlobalBoolean {
        global_input_id: String,
        value: bool,
    },
    SetGlobalVector {
        global_input_id: String,
        value: [StringNumber; 2],
    },
    Fire {
        input_name: String,
    },
    Reset {
        input_name: String,
    },
    SetTheme {
        value: String,
    },
    SetFrame {
        value: StringNumber,
    },
    SetProgress {
        value: StringNumber,
    },
    FireCustomEvent {
        value: String,
    },
}

impl ActionTrait for Action {
    fn execute(
        &self,
        engine: &mut StateMachineEngine,
        player: Rc<RwLock<DotLottiePlayerContainer>>,
        run_pipeline: bool,
        called_from_action: bool,
    ) -> Result<(), StateMachineActionError> {
        match self {
            Action::Increment { input_name, value } => {
                let current_val = engine.inputs.resolve_numeric(&input_name).ok_or_else(|| {
                    return StateMachineActionError::ExecuteError;
                })?;

                let increment_amount = match value {
                    Some(StringNumber::String(ref name)) => {
                        engine.inputs.resolve_numeric(name).unwrap_or(1.0)
                    }
                    Some(StringNumber::F32(v)) => *v,
                    None => 1.0,
                };

                let new_value = current_val + increment_amount;

                engine.set_numeric_input(input_name, new_value, run_pipeline, called_from_action);

                Ok(())
            }
            Action::Decrement { input_name, value } => {
                let current_val = engine.inputs.resolve_numeric(&input_name).ok_or_else(|| {
                    return StateMachineActionError::ExecuteError;
                })?;

                let increment_amount = match value {
                    Some(StringNumber::String(ref name)) => {
                        engine.inputs.resolve_numeric(name).unwrap_or(1.0)
                    }
                    Some(StringNumber::F32(v)) => *v,
                    None => 1.0,
                };

                let new_value = current_val - increment_amount;

                engine.set_numeric_input(input_name, new_value, run_pipeline, called_from_action);

                Ok(())
            }
            Action::Toggle { input_name } => {
                if let Some(val) = engine.get_boolean_input(input_name) {
                    engine.set_boolean_input(input_name, !val, run_pipeline, called_from_action);
                }

                Ok(())
            }
            Action::SetBoolean { input_name, value } => {
                let new_value = match value {
                    StringBool::String(ref name) => engine
                        .inputs
                        .resolve_boolean(name)
                        .ok_or(StateMachineActionError::ExecuteError)?,
                    StringBool::Bool(v) => *v,
                };

                engine.set_boolean_input(input_name, new_value, run_pipeline, called_from_action);

                Ok(())
            }
            Action::SetNumeric { input_name, value } => {
                let new_value = match value {
                    StringNumber::String(ref name) => engine
                        .inputs
                        .resolve_numeric(name)
                        .ok_or(StateMachineActionError::ExecuteError)?,
                    StringNumber::F32(v) => *v,
                };

                engine.set_numeric_input(input_name, new_value, run_pipeline, called_from_action);

                Ok(())
            }
            Action::SetString { input_name, value } => {
                let new_value = engine
                    .inputs
                    .resolve_string(input_name)
                    .unwrap_or_else(|| value.clone());

                engine.set_string_input(input_name, &new_value, run_pipeline, called_from_action);

                Ok(())
            }
            Action::Fire { input_name } => {
                let new_value = engine
                    .inputs
                    .resolve_string(input_name)
                    .unwrap_or_else(|| input_name.clone());

                let _ = engine.fire(&new_value, run_pipeline);
                Ok(())
            }
            Action::Reset { input_name } => {
                let new_value = engine
                    .inputs
                    .resolve_string(input_name)
                    .unwrap_or_else(|| input_name.clone());

                engine.reset_input(&new_value, run_pipeline, called_from_action);

                Ok(())
            }
            Action::SetTheme { value } => {
                let player = player
                    .try_read()
                    .map_err(|_| StateMachineActionError::ExecuteError)?;

                let resolved_value = engine
                    .inputs
                    .resolve_string(value)
                    .unwrap_or_else(|| value.clone());

                if !player.set_theme(&resolved_value) {
                    return Err(StateMachineActionError::ExecuteError);
                }

                Ok(())
            }
            Action::OpenUrl { url, target } => {
                let resolved_url = engine
                    .inputs
                    .resolve_string(url)
                    .unwrap_or_else(|| url.clone());

                // Urls are only opened if they are strictly inside the whitelist
                if !engine
                    .open_url_whitelist
                    .is_allowed(&resolved_url)
                    .unwrap_or(false)
                {
                    return Err(StateMachineActionError::ExecuteError);
                }

                // User has configured the player to only open urls based on click or pointer down events
                if engine.open_url_requires_user_interaction {
                    match engine.pointer_management.most_recent_event {
                        Some(Event::PointerDown { .. } | Event::Click { .. }) => {}
                        _ => return Err(StateMachineActionError::ExecuteError),
                    }
                }

                let command = if target.is_empty() {
                    format!("OpenUrl: {resolved_url}")
                } else {
                    format!("OpenUrl: {resolved_url} | Target: {target}")
                };

                engine.observe_internal_event(&command);
                Ok(())
            }
            Action::FireCustomEvent { value } => {
                engine.observe_custom_event(value);

                Ok(())
            }
            Action::SetFrame { value } => {
                let player = player
                    .read()
                    .map_err(|_| StateMachineActionError::ExecuteError)?;

                let frame = match value {
                    StringNumber::String(ref name) => engine
                        .inputs
                        .resolve_numeric(name)
                        .ok_or(StateMachineActionError::ExecuteError)?,
                    StringNumber::F32(v) => *v,
                };

                let clamped_frame = frame.clamp(0.0, player.total_frames() - 1.0);
                player.set_frame(clamped_frame);

                Ok(())
            }
            Action::SetProgress { value } => {
                let player = player
                    .read()
                    .map_err(|_| StateMachineActionError::ExecuteError)?;

                let percentage = match value {
                    StringNumber::String(ref name) => engine
                        .inputs
                        .resolve_numeric(name)
                        .ok_or(StateMachineActionError::ExecuteError)?,
                    StringNumber::F32(v) => *v,
                };

                let clamped_percentage = percentage.clamp(0.0, 100.0);
                let frame = (player.total_frames() - 1.0) * (clamped_percentage / 100.0);
                player.set_frame(frame);

                Ok(())
            }
            Action::SetGlobalVector {
                global_input_id,
                value,
            } => {
                let player = player
                    .read()
                    .map_err(|_| StateMachineActionError::ExecuteError)?;

                let first_value = match value[0] {
                    StringNumber::String(ref name) => engine
                        .inputs
                        .resolve_numeric(name)
                        .ok_or(StateMachineActionError::ExecuteError)?,
                    StringNumber::F32(v) => v,
                };

                let second_value = match value[1] {
                    StringNumber::String(ref name) => engine
                        .inputs
                        .resolve_numeric(name)
                        .ok_or(StateMachineActionError::ExecuteError)?,
                    StringNumber::F32(v) => v,
                };

                player.global_inputs_set_vector(
                    &global_input_id,
                    &[first_value.into(), second_value.into()],
                );
                Ok(())
            }
            Action::SetGlobalColor {
                global_input_id,
                value,
            } => {
                let player = player
                    .read()
                    .map_err(|_| StateMachineActionError::ExecuteError)?;

                if value.len() >= 3 {
                    let first_value = match value[0] {
                        StringNumber::String(ref name) => engine
                            .inputs
                            .resolve_numeric(name)
                            .ok_or(StateMachineActionError::ExecuteError)?,
                        StringNumber::F32(v) => v,
                    };

                    let second_value = match value[1] {
                        StringNumber::String(ref name) => engine
                            .inputs
                            .resolve_numeric(name)
                            .ok_or(StateMachineActionError::ExecuteError)?,
                        StringNumber::F32(v) => v,
                    };

                    let third_value = match value[2] {
                        StringNumber::String(ref name) => engine
                            .inputs
                            .resolve_numeric(name)
                            .ok_or(StateMachineActionError::ExecuteError)?,
                        StringNumber::F32(v) => v,
                    };

                    let fourth_value = if value.len() >= 4 {
                        match &value[3] {
                            StringNumber::String(name) => engine
                                .inputs
                                .resolve_numeric(name)
                                .ok_or(StateMachineActionError::ExecuteError)?,
                            StringNumber::F32(v) => *v,
                        }
                    } else {
                        1.0
                    };
                    player.global_inputs_set_color(
                        &global_input_id,
                        &vec![first_value, second_value, third_value, fourth_value],
                    );
                }
                Ok(())
            }
            Action::SetGlobalGradient {
                global_input_id,
                value,
            } => {
                let player = player
                    .read()
                    .map_err(|_| StateMachineActionError::ExecuteError)?;

                player.global_inputs_set_gradient(&global_input_id, value);
                Ok(())
            }
            Action::SetGlobalString {
                global_input_id,
                value,
            } => {
                let player = player
                    .read()
                    .map_err(|_| StateMachineActionError::ExecuteError)?;

                player.global_inputs_set_string(&global_input_id, value);
                Ok(())
            }
            Action::SetGlobalNumeric {
                global_input_id,
                value,
            } => {
                let player = player
                    .read()
                    .map_err(|_| StateMachineActionError::ExecuteError)?;

                player.global_inputs_set_numeric(&global_input_id, *value);
                Ok(())
            }
            Action::SetGlobalBoolean {
                global_input_id,
                value,
            } => {
                let player = player
                    .read()
                    .map_err(|_| StateMachineActionError::ExecuteError)?;

                player.global_inputs_set_boolean(&global_input_id, *value);
                Ok(())
            }
        }
    }
}
