use std::collections::HashMap;

use crate::state_machine::parser::{StringNumberBool, TransitionGuardConditionType};

#[derive(Debug)]
pub struct Guard {
    pub context_key: String,
    pub condition_type: TransitionGuardConditionType,
    pub compare_to: StringNumberBool,
}

impl Guard {
    pub fn new(
        context_key: String,
        condition_type: TransitionGuardConditionType,
        compare_to: StringNumberBool,
    ) -> Self {
        Self {
            context_key,
            condition_type,
            compare_to,
        }
    }

    pub fn string_context_is_satisfied(&self, context: &HashMap<String, String>) -> bool {
        let context_value = context.get(&self.context_key);

        if context_value.is_none() {
            return false;
        }

        match &self.compare_to {
            StringNumberBool::String(compare_to) => match self.condition_type {
                TransitionGuardConditionType::Equal => return context_value == Some(compare_to),
                TransitionGuardConditionType::NotEqual => return context_value != Some(compare_to),
                _ => return false,
            },
            StringNumberBool::F32(_) => false,
            StringNumberBool::Bool(_) => false,
        };

        false
    }

    pub fn bool_context_is_satisfied(&self, context: &HashMap<String, bool>) -> bool {
        let context_value = context.get(&self.context_key);

        if context_value.is_none() {
            return false;
        }

        match &self.compare_to {
            StringNumberBool::Bool(compare_to) => match self.condition_type {
                TransitionGuardConditionType::Equal => {
                    return context_value == Some(compare_to);
                }
                TransitionGuardConditionType::NotEqual => return context_value != Some(compare_to),
                _ => return false,
            },
            StringNumberBool::String(_) => false,
            StringNumberBool::F32(_) => false,
        };

        false
    }

    pub fn numeric_context_is_satisfied(&self, context: &HashMap<String, f32>) -> bool {
        let context_value = context.get(&self.context_key);

        if context_value.is_none() {
            return false;
        }

        match &self.compare_to {
            StringNumberBool::F32(compare_to) => match self.condition_type {
                TransitionGuardConditionType::Equal => context_value == Some(compare_to),
                TransitionGuardConditionType::NotEqual => context_value != Some(compare_to),
                TransitionGuardConditionType::GreaterThan => context_value > Some(compare_to),
                TransitionGuardConditionType::LessThan => context_value < Some(compare_to),
                TransitionGuardConditionType::GreaterThanOrEqual => {
                    context_value >= Some(compare_to)
                }
                TransitionGuardConditionType::LessThanOrEqual => context_value <= Some(compare_to),
            },
            StringNumberBool::String(_) => false,
            StringNumberBool::Bool(_) => false,
        }
    }
}
