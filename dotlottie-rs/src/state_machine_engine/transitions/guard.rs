use std::collections::HashMap;

use serde::Deserialize;

use crate::state_machine::StringNumberBool;

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
    fn string_trigger_is_satisfied(&self, context: &HashMap<String, String>) -> bool;
    fn bool_trigger_is_satisfied(&self, context: &HashMap<String, bool>) -> bool;
    fn numeric_trigger_is_satisfied(&self, context: &HashMap<String, f32>) -> bool;
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
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
        compare_to: StringNumberBool,
    },
    Event {
        trigger_name: String,
    },
}

impl GuardTrait for Guard {
    fn bool_trigger_is_satisfied(&self, context: &HashMap<String, bool>) -> bool {
        match self {
            Guard::Boolean {
                trigger_name,
                condition_type,
                compare_to,
            } => {
                let context_value = context.get(trigger_name);

                if context_value.is_none() {
                    return false;
                }

                match compare_to {
                    StringNumberBool::Bool(compare_to) => match condition_type {
                        TransitionGuardConditionType::Equal => {
                            return context_value == Some(compare_to);
                        }
                        TransitionGuardConditionType::NotEqual => {
                            return context_value != Some(compare_to)
                        }
                        _ => return false,
                    },
                    StringNumberBool::String(_) => false,
                    StringNumberBool::F32(_) => false,
                };

                false
            }
            _ => false,
        }
    }

    fn string_trigger_is_satisfied(&self, context: &HashMap<String, String>) -> bool {
        match self {
            Guard::String {
                trigger_name,
                condition_type,
                compare_to,
            } => {
                let context_value = context.get(trigger_name);

                if context_value.is_none() {
                    return false;
                }

                match compare_to {
                    StringNumberBool::String(compare_to) => match condition_type {
                        TransitionGuardConditionType::Equal => {
                            return context_value == Some(compare_to)
                        }
                        TransitionGuardConditionType::NotEqual => {
                            return context_value != Some(compare_to)
                        }
                        _ => return false,
                    },
                    StringNumberBool::F32(_) => false,
                    StringNumberBool::Bool(_) => false,
                };

                false
            }
            _ => false,
        }
    }

    fn numeric_trigger_is_satisfied(&self, context: &HashMap<String, f32>) -> bool {
        match self {
            Guard::Boolean {
                trigger_name,
                condition_type,
                compare_to,
            } => {
                let context_value = context.get(trigger_name);

                if context_value.is_none() {
                    return false;
                }

                match compare_to {
                    StringNumberBool::F32(compare_to) => match condition_type {
                        TransitionGuardConditionType::Equal => context_value == Some(compare_to),
                        TransitionGuardConditionType::NotEqual => context_value != Some(compare_to),
                        TransitionGuardConditionType::GreaterThan => {
                            context_value > Some(compare_to)
                        }
                        TransitionGuardConditionType::LessThan => context_value < Some(compare_to),
                        TransitionGuardConditionType::GreaterThanOrEqual => {
                            context_value >= Some(compare_to)
                        }
                        TransitionGuardConditionType::LessThanOrEqual => {
                            context_value <= Some(compare_to)
                        }
                    },
                    StringNumberBool::String(_) => false,
                    StringNumberBool::Bool(_) => false,
                }
            }
            _ => false,
        }
    }
}
