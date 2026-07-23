#[cfg(feature = "theming")]
use std::ffi::CString;

use super::definition::StringBool;
use crate::json::{opt, Value};
use crate::state_machine::definition::{dot_string, string_bool, string_number};
use crate::string::{DotString, DotStringInterner};
use crate::Event;

use super::{definition::StringNumber, StateMachineEngine, GLOBAL_INPUT_PREFIX};

fn resolve_numeric_ref(engine: &StateMachineEngine, value: &str) -> Option<f32> {
    if value.starts_with(GLOBAL_INPUT_PREFIX) {
        engine.get_numeric_input(value)
    } else if value.starts_with('$') {
        engine.get_numeric_input(value.trim_start_matches('$'))
    } else {
        None
    }
}

fn resolve_clamp_bound(
    engine: &StateMachineEngine,
    bound: &Option<StringNumber>,
) -> Result<Option<f32>, ()> {
    match bound {
        None => Ok(None),
        Some(StringNumber::F32(v)) => Ok(Some(*v)),
        Some(StringNumber::String(s)) => resolve_numeric_ref(engine, s).map(Some).ok_or(()),
    }
}

fn resolve_random_bound(
    engine: &StateMachineEngine,
    bound: &Option<StringNumber>,
    default: f32,
) -> Result<f32, ()> {
    match bound {
        None => Ok(default),
        Some(StringNumber::F32(v)) => Ok(*v),
        Some(StringNumber::String(s)) => resolve_numeric_ref(engine, s).ok_or(()),
    }
}

pub mod open_url_policy;
pub mod whitelist;

#[derive(Debug, thiserror::Error)]
pub enum StateMachineActionError {
    #[error("action execution failed")]
    ExecuteError,
    #[error("action parsing failed")]
    ParsingError,
}

pub trait ActionTrait {
    fn execute(
        &self,
        engine: &mut StateMachineEngine,
        run_pipeline: bool,
        called_from_interaction: bool,
    ) -> Result<(), StateMachineActionError>;
}

#[derive(Debug, Clone)]
pub enum Action {
    OpenUrl {
        url: String,
        target: String,
    },
    Increment {
        input_name: DotString,
        value: Option<StringNumber>,
    },
    Decrement {
        input_name: DotString,
        value: Option<StringNumber>,
    },
    Toggle {
        input_name: DotString,
    },
    SetBoolean {
        input_name: DotString,
        value: StringBool,
    },
    SetString {
        input_name: DotString,
        value: String,
    },
    SetNumeric {
        input_name: DotString,
        value: StringNumber,
    },
    SetRandom {
        input_name: DotString,
        min: Option<StringNumber>,
        max: Option<StringNumber>,
        integer: Option<bool>,
    },
    Multiply {
        input_name: DotString,
        value: StringNumber,
    },
    Floor {
        input_name: DotString,
    },
    Clamp {
        input_name: DotString,
        min: Option<StringNumber>,
        max: Option<StringNumber>,
    },
    Fire {
        input_name: DotString,
    },
    Reset {
        input_name: DotString,
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

pub(crate) fn action_from_json(v: &Value) -> Option<Action> {
    let input_name = || -> Option<DotString> { dot_string(v.get("inputName")?) };
    let req_string = |key: &str| -> Option<String> { v.str_field(key).map(str::to_owned) };
    Some(match v.str_field("type")? {
        "OpenUrl" => Action::OpenUrl {
            url: req_string("url")?,
            target: req_string("target")?,
        },
        "Increment" => Action::Increment {
            input_name: input_name()?,
            value: opt(v.get("value"), string_number)?,
        },
        "Decrement" => Action::Decrement {
            input_name: input_name()?,
            value: opt(v.get("value"), string_number)?,
        },
        "Toggle" => Action::Toggle {
            input_name: input_name()?,
        },
        "SetBoolean" => Action::SetBoolean {
            input_name: input_name()?,
            value: string_bool(v.get("value")?)?,
        },
        "SetString" => Action::SetString {
            input_name: input_name()?,
            value: req_string("value")?,
        },
        "SetNumeric" => Action::SetNumeric {
            input_name: input_name()?,
            value: string_number(v.get("value")?)?,
        },
        "SetRandom" => Action::SetRandom {
            input_name: input_name()?,
            min: opt(v.get("min"), string_number)?,
            max: opt(v.get("max"), string_number)?,
            integer: opt(v.get("integer"), Value::as_bool)?,
        },
        "Multiply" => Action::Multiply {
            input_name: input_name()?,
            value: string_number(v.get("value")?)?,
        },
        "Floor" => Action::Floor {
            input_name: input_name()?,
        },
        "Clamp" => Action::Clamp {
            input_name: input_name()?,
            min: opt(v.get("min"), string_number)?,
            max: opt(v.get("max"), string_number)?,
        },
        "Fire" => Action::Fire {
            input_name: input_name()?,
        },
        "Reset" => Action::Reset {
            input_name: input_name()?,
        },
        "SetTheme" => Action::SetTheme {
            value: req_string("value")?,
        },
        "SetFrame" => Action::SetFrame {
            value: string_number(v.get("value")?)?,
        },
        "SetProgress" => Action::SetProgress {
            value: string_number(v.get("value")?)?,
        },
        "FireCustomEvent" => Action::FireCustomEvent {
            value: req_string("value")?,
        },
        _ => return None,
    })
}

impl Action {
    pub(crate) fn intern_identifiers(&mut self, interner: &mut DotStringInterner) {
        let input_name = match self {
            Action::Increment { input_name, .. }
            | Action::Decrement { input_name, .. }
            | Action::Toggle { input_name }
            | Action::SetBoolean { input_name, .. }
            | Action::SetString { input_name, .. }
            | Action::SetNumeric { input_name, .. }
            | Action::SetRandom { input_name, .. }
            | Action::Multiply { input_name, .. }
            | Action::Floor { input_name }
            | Action::Clamp { input_name, .. }
            | Action::Fire { input_name }
            | Action::Reset { input_name } => input_name,
            Action::OpenUrl { .. }
            | Action::SetTheme { .. }
            | Action::SetFrame { .. }
            | Action::SetProgress { .. }
            | Action::FireCustomEvent { .. } => return,
        };
        *input_name = interner.intern(input_name.as_str());
    }
}

impl ActionTrait for Action {
    fn execute(
        &self,
        engine: &mut StateMachineEngine,
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
                                let opt_input_value = resolve_numeric_ref(engine, value);
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
                                let opt_input_value = resolve_numeric_ref(engine, value);
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
                            let opt_input_value = resolve_numeric_ref(engine, string_value);

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
            Action::SetRandom {
                input_name,
                min,
                max,
                integer,
            } => {
                if engine.get_numeric_input(input_name).is_some() {
                    let lo = match resolve_random_bound(engine, min, 0.0) {
                        Ok(v) => v,
                        Err(()) => return Ok(()),
                    };
                    let hi = match resolve_random_bound(engine, max, 1.0) {
                        Ok(v) => v,
                        Err(()) => return Ok(()),
                    };
                    if lo > hi {
                        return Ok(());
                    }

                    let r = engine.next_random();
                    let value = if matches!(integer, Some(true)) {
                        lo + (r * (hi - lo + 1.0)).floor()
                    } else {
                        lo + r * (hi - lo)
                    };
                    engine.set_numeric_input(input_name, value, run_pipeline, called_from_action);
                }
                Ok(())
            }
            Action::Multiply { input_name, value } => {
                if let Some(val) = engine.get_numeric_input(input_name) {
                    let operand = match value {
                        StringNumber::String(string_value) => {
                            resolve_numeric_ref(engine, string_value)
                        }
                        StringNumber::F32(numeric_value) => Some(*numeric_value),
                    };
                    if let Some(operand) = operand {
                        let result = val * operand;
                        if result != val {
                            engine.set_numeric_input(
                                input_name,
                                result,
                                run_pipeline,
                                called_from_action,
                            );
                        }
                    }
                }
                Ok(())
            }
            Action::Floor { input_name } => {
                if let Some(val) = engine.get_numeric_input(input_name) {
                    let result = val.floor();
                    if result != val {
                        engine.set_numeric_input(
                            input_name,
                            result,
                            run_pipeline,
                            called_from_action,
                        );
                    }
                }
                Ok(())
            }
            Action::Clamp {
                input_name,
                min,
                max,
            } => {
                if let Some(val) = engine.get_numeric_input(input_name) {
                    // A present-but-unresolvable bound makes the whole action a
                    // no-op; an omitted bound is unbounded on that side.
                    let (lo, hi) = match (
                        resolve_clamp_bound(engine, min),
                        resolve_clamp_bound(engine, max),
                    ) {
                        (Ok(lo), Ok(hi)) => (lo, hi),
                        _ => return Ok(()),
                    };

                    // Both bounds absent -> nothing to clamp.
                    if lo.is_none() && hi.is_none() {
                        return Ok(());
                    }

                    // Inverted bounds are invalid; don't silently swap.
                    if let (Some(lo), Some(hi)) = (lo, hi) {
                        if lo > hi {
                            return Ok(());
                        }
                    }

                    let mut result = val;
                    if let Some(lo) = lo {
                        result = result.max(lo);
                    }
                    if let Some(hi) = hi {
                        result = result.min(hi);
                    }

                    if result != val {
                        engine.set_numeric_input(
                            input_name,
                            result,
                            run_pipeline,
                            called_from_action,
                        );
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
            #[cfg_attr(not(feature = "theming"), allow(unused_variables))]
            Action::SetTheme { value } => {
                #[cfg(feature = "theming")]
                {
                    let resolved_value = value
                        .strip_prefix('$')
                        .and_then(|key| engine.get_string_input(key))
                        .unwrap_or_else(|| value.clone());

                    let theme_cstr = CString::new(resolved_value)
                        .map_err(|_| StateMachineActionError::ParsingError)?;

                    engine
                        .player
                        .set_theme(&theme_cstr)
                        .map_err(|_| StateMachineActionError::ExecuteError)?;
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
            Action::SetFrame { value } => {
                match value {
                    StringNumber::String(value) => {
                        let frame = resolve_numeric_ref(engine, value);
                        if let Some(frame) = frame {
                            let clamped_frame =
                                frame.clamp(0.0, engine.player.total_frames() - 1.0);
                            let _ = engine.player.set_frame(clamped_frame);
                        } else {
                            return Err(StateMachineActionError::ExecuteError);
                        }
                        return Ok(());
                    }
                    StringNumber::F32(value) => {
                        let clamped_frame = value.clamp(0.0, engine.player.total_frames() - 1.0);
                        let _ = engine.player.set_frame(clamped_frame);
                    }
                }
                Ok(())
            }
            Action::SetProgress { value } => {
                match value {
                    StringNumber::String(value) => {
                        let percentage = resolve_numeric_ref(engine, value);
                        if let Some(percentage) = percentage {
                            let clamped_value = percentage.clamp(0.0, 100.0);
                            let new_perc = clamped_value / 100.0;
                            let frame = (engine.player.total_frames() - 1.0) * new_perc;

                            let _ = engine.player.set_frame(frame);
                        }

                        return Ok(());
                    }
                    StringNumber::F32(value) => {
                        let clamped_value = value.clamp(0.0, 100.0);
                        let new_perc = clamped_value / 100.0;
                        let frame = (engine.player.total_frames() - 1.0) * new_perc;

                        let _ = engine.player.set_frame(frame);
                    }
                }

                Ok(())
            }
        }
    }
}
