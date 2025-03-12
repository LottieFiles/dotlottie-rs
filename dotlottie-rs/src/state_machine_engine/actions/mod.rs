use open_url::OpenUrlMode;
use thiserror::Error;

use serde::Deserialize;
use utils::NativeOpenUrl;
use whitelist::Whitelist;

use std::{rc::Rc, sync::RwLock};

use crate::{DotLottiePlayerContainer, Event};

use super::{state_machine::StringNumber, StateMachineEngine};

pub mod open_url;
mod utils;
mod whitelist;

#[derive(Error, Debug)]
pub enum StateMachineActionError {
    #[error("Error executing action: {0}")]
    ExecuteError(String),
}

pub trait ActionTrait {
    fn execute(
        &self,
        engine: &mut StateMachineEngine,
        player: Rc<RwLock<DotLottiePlayerContainer>>,
        run_pipeline: bool,
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
        value: bool,
    },
    SetString {
        input_name: String,
        value: String,
    },
    SetNumeric {
        input_name: String,
        value: f32,
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
}

impl ActionTrait for Action {
    fn execute(
        &self,
        engine: &mut StateMachineEngine,
        player: Rc<RwLock<DotLottiePlayerContainer>>,
        run_pipeline: bool,
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
                                        true,
                                    );
                                } else {
                                    engine.set_numeric_input(
                                        input_name,
                                        val + 1.0,
                                        run_pipeline,
                                        true,
                                    );
                                }
                            }
                            StringNumber::F32(value) => {
                                engine.set_numeric_input(
                                    input_name,
                                    val + value,
                                    run_pipeline,
                                    true,
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
                                        true,
                                    );
                                } else {
                                    engine.set_numeric_input(
                                        input_name,
                                        val - 1.0,
                                        run_pipeline,
                                        true,
                                    );
                                }
                            }
                            StringNumber::F32(value) => {
                                engine.set_numeric_input(
                                    input_name,
                                    val - value,
                                    run_pipeline,
                                    true,
                                );
                            }
                        }
                    } else {
                        engine.set_numeric_input(input_name, val - 1.0, run_pipeline, true);
                    }
                }
                Ok(())
            }
            Action::Toggle { input_name } => {
                let val = engine.get_boolean_input(input_name);

                if let Some(val) = val {
                    engine.set_boolean_input(input_name, !val, run_pipeline, true);
                }

                Ok(())
            }
            // Todo: Add support for setting a input to a input value
            Action::SetBoolean { input_name, value } => {
                engine.set_boolean_input(input_name, *value, run_pipeline, true);

                Ok(())
            }
            // Todo: Add support for setting a input to a input value
            Action::SetNumeric { input_name, value } => {
                engine.set_numeric_input(input_name, *value, run_pipeline, true);
                Ok(())
            }
            // Todo: Add support for setting a input to a input value
            Action::SetString { input_name, value } => {
                engine.set_string_input(input_name, value, run_pipeline, true);

                Ok(())
            }
            Action::Fire { input_name } => {
                let _ = engine.fire(input_name, run_pipeline);
                Ok(())
            }
            Action::Reset { input_name } => {
                engine.reset_input(input_name, run_pipeline, true);

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
                        if !player.set_theme(value) {
                            return Err(StateMachineActionError::ExecuteError(format!(
                                "Error loading theme: {}",
                                value
                            )));
                        }
                    }
                    Err(_) => {
                        return Err(StateMachineActionError::ExecuteError(
                            "Error getting read lock on player".to_string(),
                        ));
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
                            return Err(StateMachineActionError::ExecuteError(format!(
                                "Error loading theme data: {}",
                                value
                            )));
                        }
                    }
                    Err(_) => {
                        return Err(StateMachineActionError::ExecuteError(
                            "Error getting read lock on player".to_string(),
                        ));
                    }
                }

                Ok(())
            }
            Action::OpenUrl { url, target } => {
                let interaction = &engine.pointer_management.most_recent_event;

                let open_url_config = &engine.open_url_config;

                // If theres a whitelist and the url isn't present, do nothing
                if open_url_config.whitelist.len() != 0 {
                    let mut whitelist = Whitelist::new();

                    // Add patterns to whitelist
                    for entry in &open_url_config.whitelist {
                        let _ = whitelist.add(&entry);
                    }

                    if let Ok(false) | Err(_) = whitelist.is_allowed(&url) {
                        return Err(StateMachineActionError::ExecuteError(
                            "URL contained inside the Action has not been whitelisted.".to_string(),
                        ));
                    }
                }

                match open_url_config.mode {
                    OpenUrlMode::Deny => {
                        return Err(StateMachineActionError::ExecuteError(
                            "Opening URLs has been denied by the player's configuration."
                                .to_string(),
                        ));
                    }
                    OpenUrlMode::Interaction => {
                        if let Some(event) = interaction {
                            if let Event::PointerDown { .. } = event {
                                let _ =
                                    NativeOpenUrl::open_url(url, target, &engine, player.clone());
                                return Ok(());
                            }
                        }
                    }
                    OpenUrlMode::Allow => {
                        let _ = NativeOpenUrl::open_url(url, target, &engine, player.clone());
                        return Ok(());
                    }
                }

                Ok(())
            }
            Action::FireCustomEvent { value } => {
                engine.observe_custom_event(value);

                Ok(())
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
                                player.set_frame(frame);
                            } else {
                                return Err(StateMachineActionError::ExecuteError(
                                    "Error getting value from input.".to_string(),
                                ));
                            }
                            return Ok(());
                        } else {
                            return Err(StateMachineActionError::ExecuteError(
                                "Error getting read lock on player".to_string(),
                            ));
                        }
                    }
                    StringNumber::F32(value) => {
                        if let Ok(player) = read_lock {
                            player.set_frame(*value);
                        } else {
                            return Err(StateMachineActionError::ExecuteError(
                                "Error getting read lock on player".to_string(),
                            ));
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
                                    let new_perc = percentage / 100.0;
                                    let frame = player.total_frames() * new_perc;
                                    player.set_frame(frame);
                                }

                                return Ok(());
                            }
                            StringNumber::F32(value) => {
                                let new_perc = value / 100.0;
                                let frame = player.total_frames() * new_perc;
                                player.set_frame(frame);
                            }
                        }
                    }
                    Err(_) => {
                        return Err(StateMachineActionError::ExecuteError(
                            "Error getting read lock on player".to_string(),
                        ));
                    }
                }

                Ok(())
            }
        }
    }
}
