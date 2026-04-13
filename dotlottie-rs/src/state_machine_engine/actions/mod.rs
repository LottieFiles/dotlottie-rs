#[cfg(feature = "theming")]
use std::ffi::CString;

use serde::Deserialize;

use crate::{
    guard::{self, Guard, GuardTrait},
    inputs::InputManager,
    state_machine::StringBool,
    Event,
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
        run_pipeline: bool,
        called_from_interaction: bool,
    ) -> Result<(), StateMachineActionError>;

    fn guards(&self) -> &Option<Vec<Guard>>;
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "type")]
pub enum Action {
    OpenUrl {
        url: String,
        target: String,
        guards: Option<Vec<Guard>>,
    },
    Increment {
        input_name: String,
        value: Option<StringNumber>,
        guards: Option<Vec<Guard>>,
    },
    Decrement {
        input_name: String,
        value: Option<StringNumber>,
        guards: Option<Vec<Guard>>,
    },
    Toggle {
        input_name: String,
        guards: Option<Vec<Guard>>,
    },
    SetBoolean {
        input_name: String,
        value: StringBool,
        guards: Option<Vec<Guard>>,
    },
    SetString {
        input_name: String,
        value: String,
        guards: Option<Vec<Guard>>,
    },
    SetNumeric {
        input_name: String,
        value: StringNumber,
        guards: Option<Vec<Guard>>,
    },
    Fire {
        input_name: String,
        guards: Option<Vec<Guard>>,
    },
    Reset {
        input_name: String,
        guards: Option<Vec<Guard>>,
    },
    SetTheme {
        value: String,
        guards: Option<Vec<Guard>>,
    },
    SetFrame {
        value: StringNumber,
        guards: Option<Vec<Guard>>,
    },
    SetProgress {
        value: StringNumber,
        guards: Option<Vec<Guard>>,
    },
    FireCustomEvent {
        value: String,
        guards: Option<Vec<Guard>>,
    },
}

impl ActionTrait for Action {
    fn execute(
        &self,
        engine: &mut StateMachineEngine,
        run_pipeline: bool,
        called_from_action: bool,
    ) -> Result<(), StateMachineActionError> {
        match self {
            Action::Increment {
                input_name,
                value,
                guards,
            } => {
                let val = engine.get_numeric_input(input_name);

                if let Some(guards) = guards {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if engine.curr_event.as_ref().is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = engine.curr_event.as_ref() {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !all_guards_satisfied {
                        return Ok(());
                    }
                }

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
            Action::Decrement {
                input_name,
                value,
                guards,
            } => {
                let val = engine.get_numeric_input(input_name);

                if let Some(guards) = guards {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if engine.curr_event.as_ref().is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = engine.curr_event.as_ref() {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !all_guards_satisfied {
                        return Ok(());
                    }
                }

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
            Action::Toggle { input_name, guards } => {
                let val = engine.get_boolean_input(input_name);

                if let Some(guards) = guards {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if engine.curr_event.as_ref().is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = engine.curr_event.as_ref() {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !all_guards_satisfied {
                        return Ok(());
                    }
                }

                if let Some(val) = val {
                    engine.set_boolean_input(input_name, !val, run_pipeline, called_from_action);
                }

                Ok(())
            }
            Action::SetBoolean {
                input_name,
                value,
                guards,
            } => {
                let val = engine.get_boolean_input(input_name);

                if let Some(guards) = guards {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if engine.curr_event.as_ref().is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = engine.curr_event.as_ref() {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !all_guards_satisfied {
                        return Ok(());
                    }
                }

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
            Action::SetNumeric {
                input_name,
                value,
                guards,
            } => {
                let val = engine.get_numeric_input(input_name);

                if let Some(guards) = guards {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if engine.curr_event.as_ref().is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = engine.curr_event.as_ref() {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !all_guards_satisfied {
                        return Ok(());
                    }
                }

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
            Action::SetString {
                input_name,
                value,
                guards,
            } => {
                let val = engine.get_string_input(input_name);

                if let Some(guards) = guards {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if engine.curr_event.as_ref().is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = engine.curr_event.as_ref() {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !all_guards_satisfied {
                        return Ok(());
                    }
                }

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
            Action::Fire { input_name, guards } => {
                if let Some(guards) = guards {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if engine.curr_event.as_ref().is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = engine.curr_event.as_ref() {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !all_guards_satisfied {
                        return Ok(());
                    }
                }
                let _ = engine.fire(input_name, run_pipeline);
                Ok(())
            }
            Action::Reset { input_name, guards } => {
                if let Some(guards) = guards {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if engine.curr_event.as_ref().is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = engine.curr_event.as_ref() {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !all_guards_satisfied {
                        return Ok(());
                    }
                }

                engine.reset_input(input_name, run_pipeline, called_from_action);

                Ok(())
            }
            #[cfg_attr(not(feature = "theming"), allow(unused_variables))]
            Action::SetTheme { value, guards } => {
                if let Some(guards) = guards {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if engine.curr_event.as_ref().is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = engine.curr_event.as_ref() {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !all_guards_satisfied {
                        return Ok(());
                    }
                }

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
            Action::OpenUrl {
                url,
                target,
                guards,
            } => {
                if let Some(guards) = guards {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if engine.curr_event.as_ref().is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = engine.curr_event.as_ref() {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !all_guards_satisfied {
                        return Ok(());
                    }
                }

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
            Action::FireCustomEvent { value, guards } => {
                if let Some(guards) = guards {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if engine.curr_event.as_ref().is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = engine.curr_event.as_ref() {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !all_guards_satisfied {
                        return Ok(());
                    }
                }

                engine.observe_custom_event(value);

                Ok(())
            }
            Action::SetFrame { value, guards } => {
                if let Some(guards) = guards {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if engine.curr_event.as_ref().is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = engine.curr_event.as_ref() {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !all_guards_satisfied {
                        return Ok(());
                    }
                }

                match value {
                    StringNumber::String(value) => {
                        // Get the frame number from the input
                        // Remove the "$" prefix from the value
                        let value = value.trim_start_matches('$');
                        let frame = engine.get_numeric_input(value);
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
            Action::SetProgress { value, guards } => {
                if let Some(guards) = guards {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&engine.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if engine.curr_event.as_ref().is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = engine.curr_event.as_ref() {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !all_guards_satisfied {
                        return Ok(());
                    }
                }

                match value {
                    StringNumber::String(value) => {
                        // Get the frame number from the input
                        // Remove the "$" prefix from the value
                        let value = value.trim_start_matches('$');
                        let percentage = engine.get_numeric_input(value);
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

    fn guards(&self) -> &Option<Vec<Guard>> {
        match self {
            Action::OpenUrl { guards, .. } => guards,
            Action::Increment {
                input_name,
                value,
                guards,
            } => guards,
            Action::Decrement {
                input_name,
                value,
                guards,
            } => guards,
            Action::Toggle { input_name, guards } => guards,
            Action::SetBoolean {
                input_name,
                value,
                guards,
            } => guards,
            Action::SetString {
                input_name,
                value,
                guards,
            } => guards,
            Action::SetNumeric {
                input_name,
                value,
                guards,
            } => guards,
            Action::Fire { input_name, guards } => guards,
            Action::Reset { input_name, guards } => guards,
            Action::SetTheme { value, guards } => guards,
            Action::SetFrame { value, guards } => guards,
            Action::SetProgress { value, guards } => guards,
            Action::FireCustomEvent { value, guards } => guards,
        }
    }
}
