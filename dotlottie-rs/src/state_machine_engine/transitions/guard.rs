use serde::Deserialize;

use crate::{
    state_machine::{StringBool, StringNumberBool},
    triggers::{TriggerManager, TriggerTrait},
};

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub enum TransitionGuardConditionType {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

pub trait GuardTrait {
    fn string_trigger_is_satisfied(&self, triggers: &TriggerManager) -> bool;
    fn boolean_trigger_is_satisfied(&self, triggers: &TriggerManager) -> bool;
    fn numeric_trigger_is_satisfied(&self, triggers: &TriggerManager) -> bool;
    fn event_trigger_is_satisfied(&self, event: &str) -> bool;
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "type")]
pub enum Guard {
    Numeric {
        trigger_name: String,
        condition_type: TransitionGuardConditionType,
        compare_to: StringNumberBool,
    },
    String {
        trigger_name: String,
        condition_type: TransitionGuardConditionType,
        compare_to: StringNumberBool,
    },
    Boolean {
        trigger_name: String,
        condition_type: TransitionGuardConditionType,
        compare_to: StringBool,
    },
    Event {
        trigger_name: String,
    },
}

impl GuardTrait for Guard {
    // Check if the trigger is satisfied
    // If the user uses compare_to as a string and pass "$" as a prefix, we use the trigger value
    // If the trigger_value is not found, we return false
    fn boolean_trigger_is_satisfied(&self, trigger: &TriggerManager) -> bool {
        match self {
            Guard::Boolean {
                trigger_name,
                condition_type,
                compare_to,
            } => {
                if let Some(trigger_value) = trigger.get_boolean(trigger_name) {
                    match compare_to {
                        StringBool::Bool(compare_to) => match condition_type {
                            TransitionGuardConditionType::Equal => {
                                return trigger_value == *compare_to;
                            }
                            TransitionGuardConditionType::NotEqual => {
                                return trigger_value != *compare_to;
                            }
                            _ => return false,
                        },
                        StringBool::String(compare_to) => {
                            // Get the number from the trigger
                            // Remove the "$" prefix from the value
                            let value = compare_to.trim_start_matches('$');
                            let opt_bool_value = trigger.get_boolean(value);
                            if let Some(bool_value) = opt_bool_value {
                                match condition_type {
                                    TransitionGuardConditionType::Equal => {
                                        return trigger_value == bool_value;
                                    }
                                    TransitionGuardConditionType::NotEqual => {
                                        return trigger_value != bool_value;
                                    }
                                    _ => return false,
                                }
                            }

                            // Failed to get value from triggers
                            false
                        }
                    };
                }

                // Failed to get value from triggers
                false
            }
            _ => false,
        }
    }

    fn string_trigger_is_satisfied(&self, trigger: &TriggerManager) -> bool {
        match self {
            Guard::String {
                trigger_name,
                condition_type,
                compare_to,
            } => {
                if let Some(trigger_value) = trigger.get_string(trigger_name) {
                    match compare_to {
                        StringNumberBool::String(compare_to) => {
                            let mut mut_compare_to = compare_to.clone();

                            if mut_compare_to.starts_with("$") {
                                // Get the string from the trigger
                                // Remove the "$" prefix from the value
                                let value = mut_compare_to.trim_start_matches('$');
                                let opt_string_value = trigger.get_string(value);
                                if let Some(string_value) = opt_string_value {
                                    mut_compare_to = string_value.clone();
                                } else {
                                    // Failed to get value from triggers
                                    return false;
                                }
                            }

                            match condition_type {
                                TransitionGuardConditionType::Equal => {
                                    return trigger_value == *mut_compare_to;
                                }
                                TransitionGuardConditionType::NotEqual => {
                                    return trigger_value != *mut_compare_to;
                                }
                                _ => return false,
                            }
                        }
                        StringNumberBool::F32(_) => false,
                        StringNumberBool::Bool(_) => false,
                    };
                }

                // Failed to get value from triggers
                false
            }
            _ => false,
        }
    }

    fn numeric_trigger_is_satisfied(&self, trigger: &TriggerManager) -> bool {
        match self {
            Guard::Numeric {
                trigger_name,
                condition_type,
                compare_to,
            } => {
                if let Some(trigger_value) = trigger.get_numeric(trigger_name) {
                    match compare_to {
                        StringNumberBool::String(compare_to) => {
                            if compare_to.starts_with("$") {
                                // Remove the "$" prefix from the value
                                let value = compare_to.trim_start_matches('$');
                                let opt_numeric_value = trigger.get_numeric(value);
                                if let Some(numeric_value) = opt_numeric_value {
                                    match condition_type {
                                        TransitionGuardConditionType::GreaterThan => {
                                            trigger_value > numeric_value
                                        }
                                        TransitionGuardConditionType::GreaterThanOrEqual => {
                                            trigger_value >= numeric_value
                                        }
                                        TransitionGuardConditionType::LessThan => {
                                            trigger_value < numeric_value
                                        }
                                        TransitionGuardConditionType::LessThanOrEqual => {
                                            trigger_value <= numeric_value
                                        }
                                        TransitionGuardConditionType::Equal => {
                                            trigger_value == numeric_value
                                        }
                                        TransitionGuardConditionType::NotEqual => {
                                            trigger_value != numeric_value
                                        }
                                    }
                                } else {
                                    // Failed to get value from triggers
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        StringNumberBool::F32(value) => match condition_type {
                            TransitionGuardConditionType::GreaterThan => trigger_value > *value,
                            TransitionGuardConditionType::GreaterThanOrEqual => {
                                trigger_value >= *value
                            }
                            TransitionGuardConditionType::LessThan => trigger_value < *value,
                            TransitionGuardConditionType::LessThanOrEqual => {
                                trigger_value <= *value
                            }
                            TransitionGuardConditionType::Equal => trigger_value == *value,
                            TransitionGuardConditionType::NotEqual => trigger_value != *value,
                        },
                        StringNumberBool::Bool(_) => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn event_trigger_is_satisfied(&self, event: &str) -> bool {
        match self {
            Guard::Event { trigger_name } => trigger_name == event,
            _ => false,
        }
    }
}
