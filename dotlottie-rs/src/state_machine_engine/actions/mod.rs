use thiserror::Error;

use serde::Deserialize;

use std::{collections::HashMap, rc::Rc, sync::RwLock};

use crate::DotLottiePlayerContainer;

#[derive(Error, Debug)]
pub enum StateMachineActionError {
    #[error("Error executing action: {0}")]
    ExecuteError(String),
}

pub trait ActionTrait {
    fn execute(
        &self,
        player: &Option<Rc<RwLock<DotLottiePlayerContainer>>>,
        string_trigger: &mut HashMap<String, String>,
        bool_trigger: &mut HashMap<String, bool>,
        numeric_trigger: &mut HashMap<String, f32>,
        event_trigger: &HashMap<String, String>,
    ) -> Result<(), StateMachineActionError>;
}

#[derive(Debug, Deserialize, Clone)]
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
        value: f32,
    },
    SetSlot {
        value: String,
    },
    FireCustomEvent {
        value: String,
    },
}

impl ActionTrait for Action {
    // Todo: How can we:
    // - Insert inside trigger and alert the StateMachine
    // - Fire an event to the state machine
    fn execute(
        &self,
        player: &Option<Rc<RwLock<DotLottiePlayerContainer>>>,
        string_trigger: &mut HashMap<String, String>,
        bool_trigger: &mut HashMap<String, bool>,
        numeric_trigger: &mut HashMap<String, f32>,
        event_trigger: &HashMap<String, String>,
    ) -> Result<(), StateMachineActionError> {
        match self {
            Action::Increment {
                trigger_name,
                value,
            } => {
                println!("Incrementing trigger {} by {:?}", trigger_name, value);

                if let Some(value) = value {
                    numeric_trigger.insert(
                        trigger_name.to_string(),
                        numeric_trigger.get(trigger_name).unwrap_or(&0.0) + value,
                    );
                } else {
                    numeric_trigger.insert(
                        trigger_name.to_string(),
                        numeric_trigger.get(trigger_name).unwrap_or(&0.0) + 1.0,
                    );
                }
                Ok(())
            }
            Action::Decrement {
                trigger_name,
                value,
            } => {
                println!("Decrementing trigger {} by {:?}", trigger_name, value);

                if let Some(value) = value {
                    numeric_trigger.insert(
                        trigger_name.to_string(),
                        numeric_trigger.get(trigger_name).unwrap_or(&0.0) - value,
                    );
                } else {
                    numeric_trigger.insert(
                        trigger_name.to_string(),
                        numeric_trigger.get(trigger_name).unwrap_or(&0.0) - 1.0,
                    );
                }
                Ok(())
            }
            Action::Toggle { trigger_name } => {
                println!("Toggling trigger {}", trigger_name);
                bool_trigger.insert(
                    trigger_name.to_string(),
                    !bool_trigger.get(trigger_name).unwrap_or(&false),
                );

                Ok(())
            }
            Action::SetBoolean {
                trigger_name,
                value,
            } => {
                println!("Setting trigger {} to {}", trigger_name, value);
                bool_trigger.insert(trigger_name.to_string(), *value);
                Ok(())
            }
            Action::SetNumeric {
                trigger_name,
                value,
            } => {
                println!("Setting trigger {} to {}", trigger_name, value);
                numeric_trigger.insert(trigger_name.to_string(), *value);
                Ok(())
            }
            Action::SetString {
                trigger_name,
                value,
            } => {
                println!("Setting trigger {} to {}", trigger_name, value);
                string_trigger.insert(trigger_name.to_string(), value.to_string());

                Ok(())
            }
            Action::Fire { trigger_name } => {
                println!("Firing trigger {}", trigger_name);
                Ok(())
            }
            Action::Reset { trigger_name } => {
                println!("Resetting trigger {}", trigger_name);
                Ok(())
            }
            Action::SetExpression {
                layer_name,
                property_index,
                var_name,
                value,
            } => {
                println!(
                    "Setting expression {} on layer {} property {} to {}",
                    var_name, layer_name, property_index, value
                );
                Ok(())
            }
            Action::SetTheme { theme_id } => {
                println!("Setting theme to {}", theme_id);
                Ok(())
            }
            Action::SetSlot { value } => {
                println!("Setting slot to {}", value);
                if let Some(player) = player {
                    let read_lock = player.read();

                    match read_lock {
                        Ok(player) => {
                            player.load_theme_data(&value);
                        }
                        Err(_) => {
                            return Err(StateMachineActionError::ExecuteError(
                                "Error getting read lock on player".to_string(),
                            ));
                        }
                    }
                }

                Ok(())
            }
            Action::OpenUrl { url } => {
                println!("Opening URL {}", url);
                Ok(())
            }
            Action::FireCustomEvent { value } => {
                println!("Firing custom event {}", value);

                Ok(())
            }
            Action::SetFrame { value } => {
                if let Some(player) = player {
                    let read_lock = player.read();

                    match read_lock {
                        Ok(player) => {
                            player.set_frame(*value);
                            return Ok(());
                        }
                        Err(_) => {
                            return Err(StateMachineActionError::ExecuteError(
                                "Error getting read lock on player".to_string(),
                            ));
                        }
                    }
                }

                Ok(())
            }
            Action::ThemeAction { theme_id } => {
                if let Some(player) = player {
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

                Ok(())
            }
        }
    }
}
