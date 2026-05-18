use std::collections::HashSet;

use super::{
    actions::Action,
    inputs::Input,
    interactions::InteractionTrait,
    state_machine::StringBool,
    states::StateTrait,
    transitions::{
        guard::{self, Guard},
        Transition, TransitionTrait,
    },
    StateMachineEngine, ELAPSED_TIME_KEY, ELAPSED_TIME_REF,
};

use crate::state_machine::StringNumberBool;
use crate::state_machine_engine::State::GlobalState;

#[derive(Debug)]
pub enum StateMachineEngineSecurityError {
    SecurityCheckErrorMultipleGuardlessTransitions,
    SecurityCheckErrorDuplicateStateName,
    SecurityCheckErrorInputCompareToIsWrong,
    MultipleGlobalStates,
    ReservedInputName,
    ReservedInputWriteFromAction,
}

// Rules checked:
// - All State names are unique
// - Checks every state has no more than one transitions without guards
// - Checks every guard's compareTo is a valid input
// - Checks guards using events are valid
// - Reserves the built-in `elapsedTime` input: rejects user declarations and
//   any non-Reset action that targets it.
pub fn state_machine_state_check_pipeline(
    state_machine: &StateMachineEngine,
) -> Result<(), StateMachineEngineSecurityError> {
    check_reserved_input_declaration(state_machine)?;

    let states = state_machine.state_machine.states();
    let mut name_set: HashSet<String> = HashSet::new();
    let mut has_global = false;

    for state in states {
        let state_name = state.name();
        check_actions_for_reserved_writes(state.entry_actions())?;
        check_actions_for_reserved_writes(state.exit_actions())?;

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
            // Check for existing inputs and events
            match check_guards_for_existing_inputs(state_machine, transition)
                .and_then(|_| check_guards_for_existing_events(state_machine, transition))
            {
                Ok(_) => continue,
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // Checks for multiple guardless transitions
        if count > 1 {
            return Err(
                StateMachineEngineSecurityError::SecurityCheckErrorMultipleGuardlessTransitions,
            );
        }
    }

    if let Some(interactions) = state_machine.state_machine.interactions() {
        for interaction in interactions {
            check_actions_for_reserved_writes(Some(interaction.get_actions()))?;
        }
    }

    Ok(())
}

fn check_reserved_input_declaration(
    state_machine: &StateMachineEngine,
) -> Result<(), StateMachineEngineSecurityError> {
    let Some(inputs) = state_machine.state_machine.inputs() else {
        return Ok(());
    };
    for input in inputs {
        let name = match input {
            Input::Numeric { name, .. }
            | Input::String { name, .. }
            | Input::Boolean { name, .. }
            | Input::Event { name } => name,
        };
        if name == ELAPSED_TIME_KEY {
            return Err(StateMachineEngineSecurityError::ReservedInputName);
        }
    }
    Ok(())
}

fn check_actions_for_reserved_writes(
    actions: Option<&Vec<Action>>,
) -> Result<(), StateMachineEngineSecurityError> {
    let Some(actions) = actions else {
        return Ok(());
    };
    for action in actions {
        let target = match action {
            Action::Increment { input_name, .. }
            | Action::Decrement { input_name, .. }
            | Action::Toggle { input_name }
            | Action::SetBoolean { input_name, .. }
            | Action::SetString { input_name, .. }
            | Action::SetNumeric { input_name, .. }
            | Action::Fire { input_name }
            | Action::Reset { input_name } => Some(input_name),
            Action::OpenUrl { .. }
            | Action::SetTheme { .. }
            | Action::SetFrame { .. }
            | Action::SetProgress { .. }
            | Action::FireCustomEvent { .. } => None,
        };
        if let Some(name) = target {
            if name == ELAPSED_TIME_KEY {
                return Err(StateMachineEngineSecurityError::ReservedInputWriteFromAction);
            }
        }
    }
    Ok(())
}

// Loop over every state and all their transitions
// If a guard is found and of type string
// Extract the input name and check if it exists in the inputs
// We can also check for correct type whilst we're at it.
pub fn check_guards_for_existing_inputs(
    state_machine: &StateMachineEngine,
    transition: &Transition,
) -> Result<(), StateMachineEngineSecurityError> {
    let guards = transition.guards();
    let inputs = state_machine.state_machine.inputs();

    if let Some(guards) = guards {
        for guard in guards {
            match guard {
                guard::Guard::Boolean { compare_to, .. } => {
                    if let StringBool::String(input_name) = compare_to {
                        let value = input_name.trim_start_matches('$');
                        let mut found = false;

                        if let Some(inputs) = inputs {
                            for input in inputs {
                                if let Input::Boolean { name, .. } = input {
                                    if name == value {
                                        found = true
                                    }
                                }
                            }
                        }

                        if !found {
                            return Err(StateMachineEngineSecurityError::SecurityCheckErrorInputCompareToIsWrong);
                        }
                    }
                }
                guard::Guard::Numeric { compare_to, .. } => {
                    if let StringNumberBool::String(input_name) = compare_to {
                        if input_name == ELAPSED_TIME_REF {
                            continue;
                        }
                        let value = input_name.trim_start_matches('$');
                        let mut found = false;

                        if let Some(inputs) = inputs {
                            for input in inputs {
                                if let Input::Numeric { name, .. } = input {
                                    if name == value {
                                        found = true
                                    }
                                }
                            }
                        }

                        if !found {
                            return Err(StateMachineEngineSecurityError::SecurityCheckErrorInputCompareToIsWrong);
                        }
                    }
                }
                guard::Guard::String { compare_to, .. } => {
                    if let StringNumberBool::String(input_name) = compare_to {
                        if input_name.starts_with("$") {
                            let value = input_name.trim_start_matches('$');
                            let mut found = false;

                            if let Some(inputs) = inputs {
                                for input in inputs {
                                    if let Input::String { name, .. } = input {
                                        if name == value {
                                            found = true
                                        }
                                    }
                                }
                            }

                            if !found {
                                return Err(StateMachineEngineSecurityError::SecurityCheckErrorInputCompareToIsWrong);
                            }
                        }
                    }
                }
                guard::Guard::Event { .. } => {
                    continue;
                }
            }
        }
    }

    Ok(())
}

pub fn check_guards_for_existing_events(
    state_machine: &StateMachineEngine,
    transition: &Transition,
) -> Result<(), StateMachineEngineSecurityError> {
    let inputs = state_machine.state_machine.inputs();

    let guards = transition.guards();

    if let Some(guards) = guards {
        for guard in guards {
            if let Guard::Event { input_name } = guard {
                let mut found = false;

                if let Some(inputs) = inputs {
                    for input in inputs {
                        if let Input::Event { name } = input {
                            if input_name == name.as_str() {
                                found = true;
                            }
                        }
                    }
                }

                if !found {
                    return Err(
                        StateMachineEngineSecurityError::SecurityCheckErrorInputCompareToIsWrong,
                    );
                }
            }
        }
    }

    Ok(())
}
