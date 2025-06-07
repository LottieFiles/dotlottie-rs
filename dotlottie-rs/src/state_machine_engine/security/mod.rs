use std::collections::HashSet;

use super::{
    inputs::Input,
    state_machine::StringBool,
    states::StateTrait,
    transitions::{
        guard::{self, Guard},
        Transition, TransitionTrait,
    },
    StateMachineEngine,
};

use crate::state_machine::StringNumberBool;
use crate::state_machine_engine::State::GlobalState;

#[derive(Debug)]
pub enum StateMachineEngineSecurityError {
    SecurityCheckErrorMultipleGuardlessTransitions,
    SecurityCheckErrorDuplicateStateName,
    SecurityCheckErrorInputCompareToIsWrong,
    MultipleGlobalStates,
}

// Rules checked:
// - All State names are unique
// - Checks every state has no more than one transitions without guards
// - Checks every guard's compareTo is a valid input
// - Checks guards using events are valid
pub fn state_machine_state_check_pipeline(
    state_machine: &StateMachineEngine,
) -> Result<(), StateMachineEngineSecurityError> {
    let states = state_machine.state_machine.states();
    let mut name_set: HashSet<String> = HashSet::new();
    let mut has_global = false;

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
                            if name == input_name {
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
