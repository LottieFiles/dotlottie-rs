use std::collections::HashSet;

use super::{
    state_machine::StringBool,
    states::StateTrait,
    transitions::{
        guard::{self, Guard},
        Transition, TransitionTrait,
    },
    triggers::Trigger,
    StateMachineEngine,
};

use crate::state_machine::StringNumberBool;

#[derive(Debug, thiserror::Error)]
pub enum StateMachineEngineSecurityError {
    #[error(
        "The state: {} has multiple transitions without guards. This is not allowed.",
        state_name
    )]
    SecurityCheckErrorMultipleGuardlessTransitions { state_name: String },

    #[error(
        "The state name: {} has been used multiple times. This is not allowed.",
        state_name
    )]
    SecurityCheckErrorDuplicateStateName { state_name: String },

    #[error(
        "A guard is using a trigger: {} for its compareTo that does not exist or is of the wrong type. This is not allowed.",
        trigger_name
    )]
    SecurityCheckErrorTriggerCompareToIsWrong { trigger_name: String },
}

// Rules checked:
// - All State names are unique
// - Checks every state has no more than one transitions without guards
// - Checks every guard's compareTo is a valid trigger
// - Checks guards using events are valid
pub fn state_machine_state_check_pipeline(
    state_machine: &StateMachineEngine,
) -> Result<(), StateMachineEngineSecurityError> {
    let states = state_machine.state_machine.states();
    let mut name_set: HashSet<String> = HashSet::new();

    for state in states {
        let state_name = state.name();

        // Check if the state names are unique
        if !name_set.insert(state_name.to_string()) {
            return Err(
                StateMachineEngineSecurityError::SecurityCheckErrorDuplicateStateName {
                    state_name: state_name.to_string(),
                },
            );
        }

        let transitions = state.transitions();
        let mut count = 0;

        for transition in transitions {
            let guards = transition.guards();

            if guards.is_none() {
                count += 1;
            } else if guards.is_some() && guards.as_ref().unwrap().is_empty() {
                count += 1;
            }

            // Check for existing triggers and events
            match check_guards_for_existing_triggers(state_machine, transition)
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
                StateMachineEngineSecurityError::SecurityCheckErrorMultipleGuardlessTransitions {
                    state_name: state.name(),
                },
            );
        }
    }

    Ok(())
}

// Loop over every state and all their transitions
// If a guard is found and of type string
// Extract the trigger name and check if it exists in the triggers
// We can also check for correct type whilst we're at it.
pub fn check_guards_for_existing_triggers(
    state_machine: &StateMachineEngine,
    transition: &Transition,
) -> Result<(), StateMachineEngineSecurityError> {
    let guards = transition.guards();
    let triggers = state_machine.state_machine.triggers();

    if let Some(guards) = guards {
        for guard in guards {
            match guard {
                guard::Guard::Boolean { compare_to, .. } => {
                    if let StringBool::String(trigger_name) = compare_to {
                        let value = trigger_name.trim_start_matches('$');
                        let mut found = false;

                        if let Some(triggers) = triggers {
                            for trigger in triggers {
                                match trigger {
                                    Trigger::Boolean { name, .. } => {
                                        if name == value {
                                            found = true
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if !found {
                            return Err(StateMachineEngineSecurityError::SecurityCheckErrorTriggerCompareToIsWrong {
                                            trigger_name: trigger_name.to_string(),
                                        });
                        }
                    }
                }
                guard::Guard::Numeric { compare_to, .. } => {
                    if let StringNumberBool::String(trigger_name) = compare_to {
                        let value = trigger_name.trim_start_matches('$');
                        let mut found = false;

                        if let Some(triggers) = triggers {
                            for trigger in triggers {
                                if let Trigger::Numeric { name, .. } = trigger {
                                    if name == value {
                                        found = true
                                    }
                                }
                            }
                        }

                        if !found {
                            return Err(StateMachineEngineSecurityError::SecurityCheckErrorTriggerCompareToIsWrong {
                                            trigger_name: trigger_name.to_string(),
                                        });
                        }
                    }
                }
                guard::Guard::String { compare_to, .. } => {
                    if let StringNumberBool::String(trigger_name) = compare_to {
                        if trigger_name.starts_with("$") {
                            let value = trigger_name.trim_start_matches('$');
                            let mut found = false;

                            if let Some(triggers) = triggers {
                                for trigger in triggers {
                                    if let Trigger::String { name, .. } = trigger {
                                        if name == value {
                                            found = true
                                        }
                                    }
                                }
                            }

                            if !found {
                                return Err(StateMachineEngineSecurityError::SecurityCheckErrorTriggerCompareToIsWrong {
                                                trigger_name: trigger_name.to_string(),
                                            });
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
    let triggers = state_machine.state_machine.triggers();

    let guards = transition.guards();

    if let Some(guards) = guards {
        for guard in guards {
            if let Guard::Event { trigger_name } = guard {
                let mut found = false;

                if let Some(triggers) = triggers {
                    for trigger in triggers {
                        if let Trigger::Event { name } = trigger {
                            if name == trigger_name {
                                found = true;
                            }
                        }
                    }
                }

                if !found {
                    return Err(
                        StateMachineEngineSecurityError::SecurityCheckErrorTriggerCompareToIsWrong {
                            trigger_name: trigger_name.to_string(),
                        },
                    );
                }
            }
        }
    }

    Ok(())
}
