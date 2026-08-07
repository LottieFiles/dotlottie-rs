use rustc_hash::FxHashSet;

use super::{
    definition::{StringBool, StringNumberBool},
    inputs::InputManager,
    transitions::{guard::Guard, Transition},
    Error, StateMachineEngine, GLOBAL_INPUT_PREFIX,
};
use crate::state_machine::State::GlobalState;

/// Load-time validation:
/// - state names are unique, and there is at most one GlobalState
/// - no state has more than one guardless transition
/// - every `$`-reference in a guard's `compareTo` names a declared input of
///   the matching type, and every event guard names a declared event
pub fn state_machine_state_check_pipeline(state_machine: &StateMachineEngine) -> Result<(), Error> {
    let inputs = &state_machine.inputs;
    let mut seen_names = FxHashSet::default();
    let mut has_global = false;

    for state in &state_machine.state_machine.states {
        if let GlobalState { .. } = state {
            if has_global {
                return Err(Error::MultipleGlobalStates);
            }
            has_global = true;
        }

        if !seen_names.insert(state.name()) {
            return Err(Error::DuplicateStateName);
        }

        let mut guardless = 0;
        for transition in state.transitions() {
            if transition.guards.is_none() {
                guardless += 1;
            }
            check_guards(inputs, transition)?;
        }

        if guardless > 1 {
            return Err(Error::MultipleGuardlessTransitions);
        }
    }

    Ok(())
}

fn check_guards(inputs: &InputManager, transition: &Transition) -> Result<(), Error> {
    for guard in transition.guards() {
        let declared = match guard {
            Guard::Boolean {
                compare_to: StringBool::String(reference),
                ..
            } => inputs
                .get_boolean(reference.trim_start_matches('$'))
                .is_some(),

            Guard::Numeric {
                compare_to: StringNumberBool::String(reference),
                ..
            } => {
                // @-prefixed refs point at built-ins (e.g. @elapsedTime)
                reference.starts_with(GLOBAL_INPUT_PREFIX)
                    || inputs
                        .get_numeric(reference.trim_start_matches('$'))
                        .is_some()
            }

            // Without a `$` prefix, `compareTo` is a literal.
            Guard::String {
                compare_to: StringNumberBool::String(reference),
                ..
            } => {
                !reference.starts_with('$')
                    || inputs
                        .get_string(reference.trim_start_matches('$'))
                        .is_some()
            }

            Guard::Event { input_name } => inputs.get_event(input_name).is_some(),

            _ => true,
        };

        if !declared {
            return Err(Error::InvalidCompareToInput);
        }
    }

    Ok(())
}
