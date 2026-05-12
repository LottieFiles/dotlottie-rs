use std::collections::HashSet;

use super::{
    inputs::Input,
    interactions::InteractionTrait,
    state_machine::StringBool,
    states::StateTrait,
    transitions::{
        guard::{self, Guard},
        TransitionTrait,
    },
    StateMachineEngine,
};

use crate::state_machine::StringNumberBool;
use crate::state_machine_engine::actions::Action;
use crate::state_machine_engine::State::GlobalState;

#[derive(Debug)]
pub enum StateMachineEngineSecurityError {
    SecurityCheckErrorMultipleGuardlessTransitions,
    SecurityCheckErrorDuplicateStateName,
    SecurityCheckErrorInputCompareToIsWrong,
    SecurityCheckErrorEventGuardOnAction,
    MultipleGlobalStates,
}

// Rules checked:
// - All State names are unique
// - At most one GlobalState
// - Each state has at most one guardless transition
// - Every transition guard's compareTo $input and Event inputName resolve to a declared Input
// - Every action guard's compareTo $input resolves to a declared Input
// - Action guards never use Guard::Event (Event guards are valid only on transitions)
pub fn state_machine_state_check_pipeline(
    state_machine: &StateMachineEngine,
) -> Result<(), StateMachineEngineSecurityError> {
    let states = state_machine.state_machine.states();
    let mut name_set: HashSet<String> = HashSet::new();
    let mut has_global = false;
    let inputs = state_machine.state_machine.inputs();

    for state in states {
        let state_name = state.name();

        if let GlobalState { .. } = state {
            if has_global {
                return Err(StateMachineEngineSecurityError::MultipleGlobalStates);
            }
            has_global = true;
        }

        // Check if the state names are unique
        if !name_set.insert(state_name.to_string()) {
            return Err(StateMachineEngineSecurityError::SecurityCheckErrorDuplicateStateName);
        }

        let transitions = state.transitions();
        let mut count = 0;

        for transition in transitions {
            let guards = transition.guards();

            if guards.is_none() {
                count += 1;
            }
            if let Some(guards) = guards {
                check_guards(guards, inputs, true)?;
            }
        }

        // Checks for multiple guardless transitions
        if count > 1 {
            return Err(
                StateMachineEngineSecurityError::SecurityCheckErrorMultipleGuardlessTransitions,
            );
        }

        if let Some(entry_actions) = state.entry_actions() {
            check_action_guards(entry_actions, inputs)?;
        }
        if let Some(exit_actions) = state.exit_actions() {
            check_action_guards(exit_actions, inputs)?;
        }
    }

    if let Some(interactions) = state_machine.state_machine.interactions() {
        for interaction in interactions {
            check_action_guards(interaction.get_actions(), inputs)?;
        }
    }

    Ok(())
}

fn check_action_guards(
    actions: &[Action],
    inputs: Option<&Vec<Input>>,
) -> Result<(), StateMachineEngineSecurityError> {
    for action in actions {
        if let Some(guards) = action.guards() {
            check_guards(guards, inputs, false)?;
        }
    }
    Ok(())
}

/// Validate a guard list against the declared inputs.
///
/// `allow_event` is true for transition guards and false for action guards;
/// when false, encountering any `Guard::Event` is a hard error.
pub fn check_guards(
    guards: &[Guard],
    inputs: Option<&Vec<Input>>,
    allow_event: bool,
) -> Result<(), StateMachineEngineSecurityError> {
    for guard in guards {
        match guard {
            guard::Guard::Boolean { compare_to, .. } => {
                if let StringBool::String(input_name) = compare_to {
                    let value = input_name.trim_start_matches('$');
                    if !input_exists(inputs, value, |i| matches!(i, Input::Boolean { name, .. } if name == value))
                    {
                        return Err(
                            StateMachineEngineSecurityError::SecurityCheckErrorInputCompareToIsWrong,
                        );
                    }
                }
            }
            guard::Guard::Numeric { compare_to, .. } => {
                if let StringNumberBool::String(input_name) = compare_to {
                    let value = input_name.trim_start_matches('$');
                    if !input_exists(inputs, value, |i| matches!(i, Input::Numeric { name, .. } if name == value))
                    {
                        return Err(
                            StateMachineEngineSecurityError::SecurityCheckErrorInputCompareToIsWrong,
                        );
                    }
                }
            }
            guard::Guard::String { compare_to, .. } => {
                if let StringNumberBool::String(input_name) = compare_to {
                    if input_name.starts_with('$') {
                        let value = input_name.trim_start_matches('$');
                        if !input_exists(inputs, value, |i| matches!(i, Input::String { name, .. } if name == value))
                        {
                            return Err(
                                StateMachineEngineSecurityError::SecurityCheckErrorInputCompareToIsWrong,
                            );
                        }
                    }
                }
            }
            guard::Guard::Event { input_name } => {
                if !allow_event {
                    return Err(StateMachineEngineSecurityError::SecurityCheckErrorEventGuardOnAction);
                }
                let needle = input_name.as_str();
                if !input_exists(inputs, needle, |i| matches!(i, Input::Event { name } if name == needle))
                {
                    return Err(
                        StateMachineEngineSecurityError::SecurityCheckErrorInputCompareToIsWrong,
                    );
                }
            }
        }
    }
    Ok(())
}

fn input_exists<F>(inputs: Option<&Vec<Input>>, _needle: &str, predicate: F) -> bool
where
    F: Fn(&Input) -> bool,
{
    inputs.is_some_and(|inputs| inputs.iter().any(predicate))
}
