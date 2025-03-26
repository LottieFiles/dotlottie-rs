use serde::Deserialize;

use crate::{
    inputs::{InputManager, InputTrait},
    state_machine::{StringBool, StringNumberBool},
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
    fn string_input_is_satisfied(&self, inputs: &InputManager) -> bool;
    fn boolean_input_is_satisfied(&self, inputs: &InputManager) -> bool;
    fn numeric_input_is_satisfied(&self, inputs: &InputManager) -> bool;
    fn event_input_is_satisfied(&self, event: &str) -> bool;
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "type")]
pub enum Guard {
    Numeric {
        input_name: String,
        condition_type: TransitionGuardConditionType,
        compare_to: StringNumberBool,
    },
    String {
        input_name: String,
        condition_type: TransitionGuardConditionType,
        compare_to: StringNumberBool,
    },
    Boolean {
        input_name: String,
        condition_type: TransitionGuardConditionType,
        compare_to: StringBool,
    },
    Event {
        input_name: String,
    },
}

impl GuardTrait for Guard {
    // Check if the input is satisfied
    // If the user uses compare_to as a string and pass "$" as a prefix, we use the input value
    // If the input_value is not found, we return false
    fn boolean_input_is_satisfied(&self, input: &InputManager) -> bool {
        match self {
            Guard::Boolean {
                input_name,
                condition_type,
                compare_to,
            } => {
                if let Some(input_value) = input.get_boolean(input_name) {
                    match compare_to {
                        StringBool::Bool(compare_to) => match condition_type {
                            TransitionGuardConditionType::Equal => {
                                return input_value == *compare_to;
                            }
                            TransitionGuardConditionType::NotEqual => {
                                return input_value != *compare_to;
                            }
                            _ => return false,
                        },
                        StringBool::String(compare_to) => {
                            // Get the number from the input
                            // Remove the "$" prefix from the value
                            let value = compare_to.trim_start_matches('$');
                            let opt_bool_value = input.get_boolean(value);
                            if let Some(bool_value) = opt_bool_value {
                                match condition_type {
                                    TransitionGuardConditionType::Equal => {
                                        return input_value == bool_value;
                                    }
                                    TransitionGuardConditionType::NotEqual => {
                                        return input_value != bool_value;
                                    }
                                    _ => return false,
                                }
                            }

                            // Failed to get value from inputs
                            false
                        }
                    };
                }

                // Failed to get value from inputs
                false
            }
            _ => false,
        }
    }

    fn string_input_is_satisfied(&self, input: &InputManager) -> bool {
        match self {
            Guard::String {
                input_name,
                condition_type,
                compare_to,
            } => {
                if let Some(input_value) = input.get_string(input_name) {
                    match compare_to {
                        StringNumberBool::String(compare_to) => {
                            let mut mut_compare_to = compare_to.clone();

                            if mut_compare_to.starts_with("$") {
                                // Get the string from the input
                                // Remove the "$" prefix from the value
                                let value = mut_compare_to.trim_start_matches('$');
                                let opt_string_value = input.get_string(value);
                                if let Some(string_value) = opt_string_value {
                                    mut_compare_to = string_value.clone();
                                } else {
                                    // Failed to get value from inputs
                                    return false;
                                }
                            }

                            match condition_type {
                                TransitionGuardConditionType::Equal => {
                                    return input_value == *mut_compare_to;
                                }
                                TransitionGuardConditionType::NotEqual => {
                                    return input_value != *mut_compare_to;
                                }
                                _ => return false,
                            }
                        }
                        StringNumberBool::F32(_) => false,
                        StringNumberBool::Bool(_) => false,
                    };
                }

                // Failed to get value from inputs
                false
            }
            _ => false,
        }
    }

    fn numeric_input_is_satisfied(&self, input: &InputManager) -> bool {
        match self {
            Guard::Numeric {
                input_name,
                condition_type,
                compare_to,
            } => {
                if let Some(input_value) = input.get_numeric(input_name) {
                    match compare_to {
                        StringNumberBool::String(compare_to) => {
                            if compare_to.starts_with("$") {
                                // Remove the "$" prefix from the value
                                let value = compare_to.trim_start_matches('$');
                                let opt_numeric_value = input.get_numeric(value);
                                if let Some(numeric_value) = opt_numeric_value {
                                    match condition_type {
                                        TransitionGuardConditionType::GreaterThan => {
                                            input_value > numeric_value
                                        }
                                        TransitionGuardConditionType::GreaterThanOrEqual => {
                                            input_value >= numeric_value
                                        }
                                        TransitionGuardConditionType::LessThan => {
                                            input_value < numeric_value
                                        }
                                        TransitionGuardConditionType::LessThanOrEqual => {
                                            input_value <= numeric_value
                                        }
                                        TransitionGuardConditionType::Equal => {
                                            input_value == numeric_value
                                        }
                                        TransitionGuardConditionType::NotEqual => {
                                            input_value != numeric_value
                                        }
                                    }
                                } else {
                                    // Failed to get value from inputs
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        StringNumberBool::F32(value) => match condition_type {
                            TransitionGuardConditionType::GreaterThan => input_value > *value,
                            TransitionGuardConditionType::GreaterThanOrEqual => {
                                input_value >= *value
                            }
                            TransitionGuardConditionType::LessThan => input_value < *value,
                            TransitionGuardConditionType::LessThanOrEqual => input_value <= *value,
                            TransitionGuardConditionType::Equal => input_value == *value,
                            TransitionGuardConditionType::NotEqual => input_value != *value,
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

    fn event_input_is_satisfied(&self, event: &str) -> bool {
        match self {
            Guard::Event { input_name } => input_name == event,
            _ => false,
        }
    }
}
