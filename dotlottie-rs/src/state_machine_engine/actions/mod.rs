use thiserror::Error;

use serde::Deserialize;

use std::{process::Command, rc::Rc, sync::RwLock};

use crate::DotLottiePlayerContainer;

use super::{state_machine::StringNumber, StateMachineEngine};

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
    },
    ThemeAction {
        theme_id: String,
    },
    Increment {
        trigger_name: String,
        value: Option<StringNumber>,
    },
    Decrement {
        trigger_name: String,
        value: Option<StringNumber>,
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
    Reset {
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
        value: StringNumber,
    },
    SetProgress {
        value: StringNumber,
    },
    SetSlot {
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
            Action::Increment {
                trigger_name,
                value,
            } => {
                let val = engine.get_numeric_trigger(trigger_name);

                if let Some(val) = val {
                    if let Some(value) = value {
                        match value {
                            StringNumber::String(value) => {
                                let trimmed_value = value.trim_start_matches('$');
                                let opt_trigger_value = engine.get_numeric_trigger(trimmed_value);
                                if let Some(trigger_value) = opt_trigger_value {
                                    engine.set_numeric_trigger(
                                        trigger_name,
                                        val + trigger_value,
                                        false,
                                    );
                                } else {
                                    engine.set_numeric_trigger(
                                        trigger_name,
                                        val + 1.0,
                                        run_pipeline,
                                    );
                                }
                            }
                            StringNumber::F32(value) => {
                                engine.set_numeric_trigger(trigger_name, val + value, run_pipeline);
                            }
                        }
                    } else {
                        engine.set_numeric_trigger(trigger_name, val + 1.0, run_pipeline);
                    }
                }

                Ok(())
            }
            Action::Decrement {
                trigger_name,
                value,
            } => {
                let val = engine.get_numeric_trigger(trigger_name);

                if let Some(val) = val {
                    if let Some(value) = value {
                        match value {
                            StringNumber::String(value) => {
                                let trimmed_value = value.trim_start_matches('$');
                                let opt_trigger_value = engine.get_numeric_trigger(trimmed_value);
                                if let Some(trigger_value) = opt_trigger_value {
                                    engine.set_numeric_trigger(
                                        trigger_name,
                                        val - trigger_value,
                                        false,
                                    );
                                } else {
                                    engine.set_numeric_trigger(
                                        trigger_name,
                                        val - 1.0,
                                        run_pipeline,
                                    );
                                }
                            }
                            StringNumber::F32(value) => {
                                engine.set_numeric_trigger(trigger_name, val - value, run_pipeline);
                            }
                        }
                    } else {
                        engine.set_numeric_trigger(trigger_name, val - 1.0, run_pipeline);
                    }
                }
                Ok(())
            }
            Action::Toggle { trigger_name } => {
                let val = engine.get_boolean_trigger(trigger_name);

                if let Some(val) = val {
                    engine.set_boolean_trigger(trigger_name, !val, run_pipeline);
                }

                Ok(())
            }
            // Todo: Add support for setting a trigger to a trigger value
            Action::SetBoolean {
                trigger_name,
                value,
            } => {
                engine.set_boolean_trigger(trigger_name, *value, run_pipeline);
                Ok(())
            }
            // Todo: Add support for setting a trigger to a trigger value
            Action::SetNumeric {
                trigger_name,
                value,
            } => {
                engine.set_numeric_trigger(trigger_name, *value, run_pipeline);
                Ok(())
            }
            // Todo: Add support for setting a trigger to a trigger value
            Action::SetString {
                trigger_name,
                value,
            } => {
                engine.set_string_trigger(trigger_name, value, run_pipeline);

                Ok(())
            }
            // Todo: Add support for setting a trigger to a trigger value
            Action::Fire { trigger_name } => {
                let _ = engine.fire(&trigger_name, run_pipeline);
                Ok(())
            }
            Action::Reset { trigger_name } => {
                todo!("Reset trigger {}", trigger_name);
                // Ok(())
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
            // Todo: Add support for setting a trigger to a trigger value
            Action::SetTheme { theme_id } => {
                let read_lock = player.try_read();

                match read_lock {
                    Ok(player) => {
                        if !player.load_theme(theme_id) {
                            return Err(StateMachineActionError::ExecuteError(format!(
                                "Error loading theme: {}",
                                theme_id
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
            Action::SetSlot { value } => {
                let read_lock = player.read();

                match read_lock {
                    Ok(player) => {
                        if !player.load_theme_data(value) {
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
            // Todo: Add support for setting a trigger to a trigger value
            Action::OpenUrl { url } => {
                Command::new("open")
                    .arg(url)
                    .spawn()
                    .expect("Failed to open URL");
                Ok(())
            }
            Action::FireCustomEvent { value } => {
                println!("Firing custom event {}", value);

                Ok(())
            }
            // Todo: Add support for setting a trigger to a trigger value
            Action::SetFrame { value } => {
                let read_lock = player.read();

                match value {
                    StringNumber::String(value) => {
                        if let Ok(player) = read_lock {
                            // Get the frame number from the trigger
                            // Remove the "$" prefix from the value
                            let value = value.trim_start_matches('$');
                            let frame = engine.get_numeric_trigger(value);
                            if let Some(frame) = frame {
                                player.set_frame(frame);
                            } else {
                                println!("Couldn't get frame from trigger");
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
            // Todo: Add support for setting a trigger to a trigger value
            Action::SetProgress { value } => {
                let read_lock = player.read();

                match read_lock {
                    Ok(player) => {
                        match value {
                            StringNumber::String(value) => {
                                // Get the frame number from the trigger
                                // Remove the "$" prefix from the value
                                let value = value.trim_start_matches('$');
                                let percentage = engine.get_numeric_trigger(value);
                                if let Some(percentage) = percentage {
                                    let new_perc = percentage / 100.0;
                                    let frame = player.total_frames() as f32 * new_perc;
                                    player.set_frame(frame);
                                }

                                return Ok(());
                            }
                            StringNumber::F32(value) => {
                                let new_perc = value / 100.0;
                                let frame = player.total_frames() as f32 * new_perc;
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
            // Todo: Add support for setting a trigger to a trigger value
            Action::ThemeAction { theme_id } => {
                let read_lock = player.read();

                match read_lock {
                    Ok(player) => {
                        if !player.load_theme(theme_id) {
                            return Err(StateMachineActionError::ExecuteError(
                                "Error loading theme".to_string(),
                            ));
                        }
                        return Ok(());
                    }
                    Err(_) => {
                        return Err(StateMachineActionError::ExecuteError(
                            "Error getting read lock on player".to_string(),
                        ))
                    }
                }
            }
        }
    }
}
