use std::collections::HashSet;

use super::{
    states::StateTrait, transitions::TransitionTrait, StateMachineEngine, StateMachineEngineError,
};

// Rules checked:
// - Checks every state has no more than one transitions without guards
// - All State names are unique
pub fn check_states_for_guardless_transitions(
    state_machine: &StateMachineEngine,
) -> Result<(), StateMachineEngineError> {
    let states = state_machine.state_machine.states();
    let mut name_set: HashSet<String> = HashSet::new();

    for state in states {
        let state_name = state.get_name();
        if !name_set.insert(state_name.to_string()) {
            println!("🚨 Error: State name: {} is not unique.", state_name);
            return Err(
                StateMachineEngineError::SecurityCheckErrorDuplicateStateName {
                    state_name: state_name.to_string(),
                },
            );
        }

        let transitions = state.get_transitions();
        let mut count = 0;

        for transition in transitions {
            let guards = transition.guards();

            if guards.is_none() {
                count += 1;
            } else if guards.is_some() && guards.as_ref().unwrap().is_empty() {
                count += 1;
            }
        }

        if count > 1 {
            return Err(
                StateMachineEngineError::SecurityCheckErrorMultipleGuardlessTransitions {
                    state_name: state.get_name(),
                },
            );
        }
    }

    Ok(())
}
