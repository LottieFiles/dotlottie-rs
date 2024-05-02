use std::collections::HashMap;

use crate::parser::{StringNumberBool, TransitionGuardConditionType};

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

        match &self.compare_to {
            StringNumberBool::String(compare_to) => match self.condition_type {
                TransitionGuardConditionType::Equal => context_value == Some(&compare_to),
                TransitionGuardConditionType::NotEqual => context_value != Some(&compare_to),
                _ => false,
            },
            StringNumberBool::F32(_) => false,
            StringNumberBool::Bool(_) => false,
        };

        false
    }

    pub fn bool_context_is_satisfied(&self, context: &HashMap<String, bool>) -> bool {
        let context_value = context.get(&self.context_key);

        match &self.compare_to {
            StringNumberBool::Bool(compare_to) => match self.condition_type {
                TransitionGuardConditionType::Equal => context_value == Some(&compare_to),
                TransitionGuardConditionType::NotEqual => context_value != Some(&compare_to),
                _ => false,
            },
            StringNumberBool::String(_) => false,
            StringNumberBool::F32(_) => false,
        };

        false
    }

    pub fn numeric_context_is_satisfied(&self, context: &HashMap<String, f32>) -> bool {
        let context_value = context.get(&self.context_key);

        if let StringNumberBool::F32(f) = &self.compare_to {
            println!("Extracted f32 value: {}", f);
        }

        match &self.compare_to {
            StringNumberBool::F32(compare_to) => match self.condition_type {
                TransitionGuardConditionType::Equal => return context_value == Some(&compare_to),
                TransitionGuardConditionType::NotEqual => {
                    return context_value != Some(&compare_to)
                }
                TransitionGuardConditionType::GreaterThan => {
                    return context_value > Some(&compare_to)
                }
                TransitionGuardConditionType::LessThan => return context_value < Some(&compare_to),
                TransitionGuardConditionType::GreaterThanOrEqual => {
                    return context_value >= Some(&compare_to)
                }
                TransitionGuardConditionType::LessThanOrEqual => {
                    return context_value <= Some(&compare_to)
                }
            },
            StringNumberBool::String(_) => false,
            StringNumberBool::Bool(_) => false,
        };

        false
    }
}
