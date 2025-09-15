use serde::Deserialize;
use std::{rc::Rc, sync::RwLock};

use crate::{state_machine::StringBool, DotLottiePlayerContainer, Event};

#[cfg(feature = "tvg-lottie-expressions")]
use crate::jerryscript::Value;

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
    Fire {
        input_name: String,
    },
    Reset {
        input_name: String,
    },
    SetExpression {
        layer_name: String,
        property_index: u32,
        var_name: String,
        value: f32,
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
    SetThemeData {
        value: String,
    },
    FireCustomEvent {
        value: String,
    },
    Eval {
        input_name: String,
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
                let val = engine.get_numeric_input(input_name);

                if let Some(val) = val {
                    if let Some(value) = value {
                        match value {
                            StringNumber::String(value) => {
                                let trimmed_value = value.trim_start_matches('$');
                                let opt_input_value = engine.get_numeric_input(trimmed_value);
                                if let Some(input_value) = opt_input_value {
                                    engine.set_numeric_input(
                                        input_name,
                                        val + input_value,
                                        run_pipeline,
                                        called_from_action,
                                    );
                                } else {
                                    engine.set_numeric_input(
                                        input_name,
                                        val + 1.0,
                                        run_pipeline,
                                        called_from_action,
                                    );
                                }
                            }
                            StringNumber::F32(value) => {
                                engine.set_numeric_input(
                                    input_name,
                                    val + value,
                                    run_pipeline,
                                    called_from_action,
                                );
                            }
                        }
                    } else {
                        engine.set_numeric_input(input_name, val + 1.0, run_pipeline, true);
                    }
                }

                Ok(())
            }
            Action::Decrement { input_name, value } => {
                let val = engine.get_numeric_input(input_name);

                if let Some(val) = val {
                    if let Some(value) = value {
                        match value {
                            StringNumber::String(value) => {
                                let trimmed_value = value.trim_start_matches('$');
                                let opt_input_value = engine.get_numeric_input(trimmed_value);
                                if let Some(input_value) = opt_input_value {
                                    engine.set_numeric_input(
                                        input_name,
                                        val - input_value,
                                        run_pipeline,
                                        called_from_action,
                                    );
                                } else {
                                    engine.set_numeric_input(
                                        input_name,
                                        val - 1.0,
                                        run_pipeline,
                                        called_from_action,
                                    );
                                }
                            }
                            StringNumber::F32(value) => {
                                engine.set_numeric_input(
                                    input_name,
                                    val - value,
                                    run_pipeline,
                                    called_from_action,
                                );
                            }
                        }
                    } else {
                        engine.set_numeric_input(
                            input_name,
                            val - 1.0,
                            run_pipeline,
                            called_from_action,
                        );
                    }
                }
                Ok(())
            }
            Action::Toggle { input_name } => {
                let val = engine.get_boolean_input(input_name);

                if let Some(val) = val {
                    engine.set_boolean_input(input_name, !val, run_pipeline, called_from_action);
                }

                Ok(())
            }
            Action::SetBoolean { input_name, value } => {
                let val = engine.get_boolean_input(input_name);

                if val.is_some() {
                    match value {
                        StringBool::String(string_value) => {
                            let trimmed_value = string_value.trim_start_matches('$');
                            let opt_input_value = engine.get_boolean_input(trimmed_value);

                            // In case of failure, don't change the input_name's value
                            if let Some(input_value) = opt_input_value {
                                engine.set_boolean_input(
                                    input_name,
                                    input_value,
                                    run_pipeline,
                                    called_from_action,
                                );
                            }
                        }
                        StringBool::Bool(bool_value) => {
                            engine.set_boolean_input(
                                input_name,
                                *bool_value,
                                run_pipeline,
                                called_from_action,
                            );
                        }
                    }
                }
                Ok(())
            }
            Action::SetNumeric { input_name, value } => {
                let val = engine.get_numeric_input(input_name);

                if val.is_some() {
                    match value {
                        StringNumber::String(string_value) => {
                            let trimmed_value = string_value.trim_start_matches('$');
                            let opt_input_value = engine.get_numeric_input(trimmed_value);

                            // In case of failure, don't change the input_name's value
                            if let Some(input_value) = opt_input_value {
                                engine.set_numeric_input(
                                    input_name,
                                    input_value,
                                    run_pipeline,
                                    called_from_action,
                                );
                            }
                        }
                        StringNumber::F32(numeric_value) => {
                            engine.set_numeric_input(
                                input_name,
                                *numeric_value,
                                run_pipeline,
                                called_from_action,
                            );
                        }
                    }
                }
                Ok(())
            }
            Action::SetString { input_name, value } => {
                let val = engine.get_string_input(input_name);

                if val.is_some() {
                    let trimmed_value = value.trim_start_matches('$');
                    let opt_input_value = engine.get_string_input(trimmed_value);
                    if let Some(input_value) = opt_input_value {
                        engine.set_string_input(
                            input_name,
                            &input_value,
                            run_pipeline,
                            called_from_action,
                        );
                    } else {
                        engine.set_string_input(
                            input_name,
                            value,
                            run_pipeline,
                            called_from_action,
                        );
                    }
                }
                Ok(())
            }
            Action::Fire { input_name } => {
                let _ = engine.fire(input_name, run_pipeline);
                Ok(())
            }
            Action::Reset { input_name } => {
                engine.reset_input(input_name, run_pipeline, called_from_action);

                Ok(())
            }
            Action::SetExpression {
                layer_name,
                property_index,
                var_name,
                value,
            } => {
                todo!(
                    "Set expression for layer {} property {} var {} value {}",
                    layer_name,
                    property_index,
                    var_name,
                    value
                );
                // Ok(())
            }
            Action::SetTheme { value } => {
                let read_lock = player.try_read();

                match read_lock {
                    Ok(player) => {
                        let resolved_value = if value.starts_with('$') {
                            let trimmed_value = value.trim_start_matches('$');
                            engine
                                .get_string_input(trimmed_value)
                                .unwrap_or_else(|| value.clone())
                        } else {
                            value.clone()
                        };

                        if !player.set_theme(&resolved_value) {
                            return Err(StateMachineActionError::ExecuteError);
                        }
                    }
                    Err(_) => {
                        return Err(StateMachineActionError::ExecuteError);
                    }
                }
                Ok(())
            }
            Action::SetThemeData { value } => {
                let read_lock = player.read();

                match read_lock {
                    Ok(player) => {
                        // If there is a $x inside value, replace with the value of x
                        // If there is a $y inside value, replace with the value of x
                        let value = value
                            .replace("$x", &engine.pointer_management.pointer_x.to_string())
                            .replace("$y", &engine.pointer_management.pointer_y.to_string());

                        if !player.set_slots(&value) {
                            return Err(StateMachineActionError::ExecuteError);
                        }
                    }
                    Err(_) => {
                        return Err(StateMachineActionError::ExecuteError);
                    }
                }

                Ok(())
            }
            Action::OpenUrl { url, target } => {
                let whitelist = &engine.open_url_whitelist;
                let user_interaction_required = &engine.open_url_requires_user_interaction;

                let resolved_url = if url.starts_with('$') {
                    let trimmed_value = url.trim_start_matches('$');
                    engine
                        .get_string_input(trimmed_value)
                        .unwrap_or_else(|| url.clone())
                } else {
                    url.clone()
                };

                // Urls are only opened if they are strictly inside the whitelist
                if let Ok(false) | Err(_) = whitelist.is_allowed(&resolved_url) {
                    return Err(StateMachineActionError::ExecuteError);
                }

                let _ = target.to_lowercase();
                let command = if target.is_empty() {
                    format!("OpenUrl: {resolved_url}")
                } else {
                    format!("OpenUrl: {resolved_url} | Target: {target}")
                };

                // User has configured the player to only open urls based on click or pointer down events
                if *user_interaction_required {
                    let interaction = &engine.pointer_management.most_recent_event;

                    if let Some(Event::PointerDown { .. } | Event::Click { .. }) = interaction {
                        engine.observe_internal_event(&command);

                        return Ok(());
                    }
                    return Err(StateMachineActionError::ExecuteError);
                }

                engine.observe_internal_event(&command);

                Ok(())
            }
            Action::FireCustomEvent { value } => {
                engine.observe_custom_event(value);

                Ok(())
            }
            #[cfg(feature = "tvg-lottie-expressions")]
            Action::Eval { input_name, value } => {
                // Get or initialize the persistent JavaScript context with current input values
                let ctx = match engine.get_js_context() {
                    Ok(ctx) => ctx,
                    Err(_) => return Err(StateMachineActionError::ExecuteError),
                };

                // Evaluate the expression
                let result = match ctx.eval(value) {
                    Ok(result) => result,
                    Err(_) => return Err(StateMachineActionError::ExecuteError),
                };

                // Determine the target input type and validate/coerce result

                if let Some(_) = engine.get_numeric_input(input_name) {
                    // Target is numeric input
                    if result.is_number() {
                        let numeric_result = result.to_number();
                        if numeric_result.is_finite() {
                            engine.set_numeric_input(
                                input_name,
                                numeric_result,
                                run_pipeline,
                                called_from_action,
                            );
                        } else {
                            return Err(StateMachineActionError::ExecuteError);
                        }
                    } else {
                        return Err(StateMachineActionError::ExecuteError);
                    }
                } else if let Some(_) = engine.get_boolean_input(input_name) {
                    // Target is boolean input
                    if result.is_boolean()
                        || (!result.is_number()
                            && !result.is_string()
                            && !result.is_object()
                            && !result.is_undefined()
                            && !result.is_exception())
                    {
                        // Convert to boolean using JavaScript truthiness rules
                        let bool_result = if result.is_boolean() {
                            result.to_number() != 0.0
                        } else {
                            // Handle other boolean-like results (e.g., comparison operators)
                            result.to_number() != 0.0
                        };
                        engine.set_boolean_input(
                            input_name,
                            bool_result,
                            run_pipeline,
                            called_from_action,
                        );
                    } else {
                        return Err(StateMachineActionError::ExecuteError);
                    }
                } else if let Some(_) = engine.get_string_input(input_name) {
                    // Target is string input
                    if result.is_string() {
                        match result.to_string() {
                            Ok(string_result) => {
                                engine.set_string_input(
                                    input_name,
                                    &string_result,
                                    run_pipeline,
                                    called_from_action,
                                );
                            }
                            Err(_) => return Err(StateMachineActionError::ExecuteError),
                        }
                    } else {
                        return Err(StateMachineActionError::ExecuteError);
                    }
                } else {
                    // Input not found
                    return Err(StateMachineActionError::ExecuteError);
                }

                Ok(())
            }
            #[cfg(not(feature = "tvg-lottie-expressions"))]
            Action::Eval { .. } => {
                // Eval action requires tvg-lottie-expressions feature
                Err(StateMachineActionError::ExecuteError)
            }
            Action::SetFrame { value } => {
                let read_lock = player.read();

                match value {
                    StringNumber::String(value) => {
                        if let Ok(player) = read_lock {
                            // Get the frame number from the input
                            // Remove the "$" prefix from the value
                            let value = value.trim_start_matches('$');
                            let frame = engine.get_numeric_input(value);
                            if let Some(frame) = frame {
                                let clamped_frame = frame.clamp(0.0, player.total_frames() - 1.0);

                                player.set_frame(clamped_frame);
                            } else {
                                return Err(StateMachineActionError::ExecuteError);
                            }
                            return Ok(());
                        } else {
                            return Err(StateMachineActionError::ExecuteError);
                        }
                    }
                    StringNumber::F32(value) => {
                        if let Ok(player) = read_lock {
                            let clamped_frame = value.clamp(0.0, player.total_frames() - 1.0);

                            player.set_frame(clamped_frame);
                        } else {
                            return Err(StateMachineActionError::ExecuteError);
                        }
                    }
                }
                Ok(())
            }
            Action::SetProgress { value } => {
                let read_lock = player.read();

                match read_lock {
                    Ok(player) => {
                        match value {
                            StringNumber::String(value) => {
                                // Get the frame number from the input
                                // Remove the "$" prefix from the value
                                let value = value.trim_start_matches('$');
                                let percentage = engine.get_numeric_input(value);
                                if let Some(percentage) = percentage {
                                    let clamped_value = percentage.clamp(0.0, 100.0);
                                    let new_perc = clamped_value / 100.0;
                                    let frame = (player.total_frames() - 1.0) * new_perc;

                                    player.set_frame(frame);
                                }

                                return Ok(());
                            }
                            StringNumber::F32(value) => {
                                let clamped_value = value.clamp(0.0, 100.0);
                                let new_perc = clamped_value / 100.0;
                                let frame = (player.total_frames() - 1.0) * new_perc;

                                player.set_frame(frame);
                            }
                        }
                    }
                    Err(_) => {
                        return Err(StateMachineActionError::ExecuteError);
                    }
                }

                Ok(())
            }
        }
    }
}
