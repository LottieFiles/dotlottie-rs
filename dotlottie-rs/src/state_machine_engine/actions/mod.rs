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
    // Todo: How can we:
    // - Insert inside trigger and alert the StateMachine
    // - Fire an event to the state machine
    fn execute(
        &self,
        engine: &mut StateMachineEngine,
        player: Rc<RwLock<DotLottiePlayerContainer>>,
    ) -> Result<(), StateMachineActionError> {
        match self {
            Action::Increment {
                trigger_name,
                value,
            } => {
                let val = engine.get_numeric_trigger(trigger_name);

                if let Some(val) = val {
                    if let Some(value) = value {
                        engine.set_numeric_trigger(trigger_name, val + value, false);
                    } else {
                        engine.set_numeric_trigger(trigger_name, val + 1.0, false);

                        println!(
                            "🚧 Updated Trigger value: {:?}",
                            engine.get_numeric_trigger(trigger_name)
                        );
                    }
                }

                Ok(())
            }
            Action::Decrement {
                trigger_name,
                value,
            } => {
                println!("Decrementing trigger {} by {:?}", trigger_name, value);

                let val = engine.get_numeric_trigger(trigger_name);

                if let Some(val) = val {
                    if let Some(value) = value {
                        engine.set_numeric_trigger(trigger_name, val - value, false);
                    } else {
                        engine.set_numeric_trigger(trigger_name, val - 1.0, false);
                    }
                }
                Ok(())
            }
            Action::Toggle { trigger_name } => {
                println!("Toggling trigger {}", trigger_name);

                let val = engine.get_boolean_trigger(trigger_name);

                if let Some(val) = val {
                    engine.set_boolean_trigger(trigger_name, !val, false);
                }

                Ok(())
            }
            Action::SetBoolean {
                trigger_name,
                value,
            } => {
                println!("Setting trigger {} to {}", trigger_name, value);

                engine.set_boolean_trigger(trigger_name, *value, false);
                Ok(())
            }
            Action::SetNumeric {
                trigger_name,
                value,
            } => {
                println!("Setting trigger {} to {}", trigger_name, value);

                engine.set_numeric_trigger(trigger_name, *value, false);
                Ok(())
            }
            Action::SetString {
                trigger_name,
                value,
            } => {
                println!("Setting trigger {} to {}", trigger_name, value);

                engine.set_string_trigger(trigger_name, value, false);

                Ok(())
            }
            Action::Fire { trigger_name } => {
                println!("Firing trigger {}", trigger_name);

                engine.fire(&trigger_name);
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
                let read_lock = player.read();

                match read_lock {
                    Ok(player) => {
                        player.load_theme_data(value);
                    }
                    Err(_) => {
                        return Err(StateMachineActionError::ExecuteError(
                            "Error getting read lock on player".to_string(),
                        ));
                    }
                }

                Ok(())
            }
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
                            // player.set_frame(*value);
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
                Ok(())
            }
        }
    }
}
