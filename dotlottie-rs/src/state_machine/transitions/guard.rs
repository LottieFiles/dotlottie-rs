use crate::json::Value;
use crate::state_machine::definition::{
    dot_string, string_bool, string_number_bool, StringBool, StringNumberBool,
};
use crate::state_machine::StateMachineEngine;
use crate::string::{DotString, DotStringInterner};

#[derive(PartialEq, Debug, Clone)]
pub enum TransitionGuardConditionType {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

fn condition_type_from_json(v: &Value) -> Option<TransitionGuardConditionType> {
    Some(match v.str_field("conditionType")? {
        "GreaterThan" => TransitionGuardConditionType::GreaterThan,
        "GreaterThanOrEqual" => TransitionGuardConditionType::GreaterThanOrEqual,
        "LessThan" => TransitionGuardConditionType::LessThan,
        "LessThanOrEqual" => TransitionGuardConditionType::LessThanOrEqual,
        "Equal" => TransitionGuardConditionType::Equal,
        "NotEqual" => TransitionGuardConditionType::NotEqual,
        _ => return None,
    })
}

/// Ordering comparison — numeric guards only.
fn compare_ord<T: PartialOrd>(condition: &TransitionGuardConditionType, a: T, b: T) -> bool {
    match condition {
        TransitionGuardConditionType::GreaterThan => a > b,
        TransitionGuardConditionType::GreaterThanOrEqual => a >= b,
        TransitionGuardConditionType::LessThan => a < b,
        TransitionGuardConditionType::LessThanOrEqual => a <= b,
        TransitionGuardConditionType::Equal => a == b,
        TransitionGuardConditionType::NotEqual => a != b,
    }
}

/// Equality comparison — string and boolean guards. Ordering never matches.
fn compare_eq<T: PartialEq>(condition: &TransitionGuardConditionType, a: T, b: T) -> bool {
    match condition {
        TransitionGuardConditionType::Equal => a == b,
        TransitionGuardConditionType::NotEqual => a != b,
        _ => false,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Guard {
    Numeric {
        input_name: DotString,
        condition_type: TransitionGuardConditionType,
        compare_to: StringNumberBool,
    },
    String {
        input_name: DotString,
        condition_type: TransitionGuardConditionType,
        compare_to: StringNumberBool,
    },
    Boolean {
        input_name: DotString,
        condition_type: TransitionGuardConditionType,
        compare_to: StringBool,
    },
    Event {
        input_name: DotString,
    },
}

pub(crate) fn guard_from_json(v: &Value) -> Option<Guard> {
    let input_name = dot_string(v.get("inputName")?)?;
    Some(match v.str_field("type")? {
        "Numeric" => Guard::Numeric {
            input_name,
            condition_type: condition_type_from_json(v)?,
            compare_to: string_number_bool(v.get("compareTo")?)?,
        },
        "String" => Guard::String {
            input_name,
            condition_type: condition_type_from_json(v)?,
            compare_to: string_number_bool(v.get("compareTo")?)?,
        },
        "Boolean" => Guard::Boolean {
            input_name,
            condition_type: condition_type_from_json(v)?,
            compare_to: string_bool(v.get("compareTo")?)?,
        },
        "Event" => Guard::Event { input_name },
        _ => return None,
    })
}

impl Guard {
    pub(crate) fn intern_identifiers(&mut self, interner: &mut DotStringInterner) {
        let input_name = match self {
            Guard::Numeric { input_name, .. }
            | Guard::String { input_name, .. }
            | Guard::Boolean { input_name, .. }
            | Guard::Event { input_name } => input_name,
        };
        *input_name = interner.intern(input_name.as_str());
    }

    /// Evaluate the guard against the engine's current inputs. `event` is the
    /// event pending on this pipeline run, if any. An unresolvable `compareTo`
    /// reference leaves the guard unsatisfied.
    pub(crate) fn is_satisfied(
        &self,
        engine: &StateMachineEngine,
        event: Option<&DotString>,
    ) -> bool {
        match self {
            Guard::Event { input_name } => event.is_some_and(|e| input_name == e),

            Guard::Numeric {
                input_name,
                condition_type,
                compare_to,
            } => {
                let Some(lhs) = engine.get_numeric_input(input_name) else {
                    return false;
                };
                let rhs = match compare_to {
                    StringNumberBool::F32(value) => Some(*value),
                    StringNumberBool::String(reference) => engine.resolve_numeric_ref(reference),
                    StringNumberBool::Bool(_) => None,
                };
                rhs.is_some_and(|rhs| compare_ord(condition_type, lhs, rhs))
            }

            Guard::String {
                input_name,
                condition_type,
                compare_to,
            } => {
                let Some(lhs) = engine.inputs.get_string(input_name) else {
                    return false;
                };
                let StringNumberBool::String(compare_to) = compare_to else {
                    return false;
                };
                let rhs = if compare_to.starts_with('$') {
                    match engine.inputs.get_string(compare_to.trim_start_matches('$')) {
                        Some(value) => value,
                        None => return false,
                    }
                } else {
                    compare_to.as_str()
                };
                compare_eq(condition_type, lhs, rhs)
            }

            Guard::Boolean {
                input_name,
                condition_type,
                compare_to,
            } => {
                let Some(lhs) = engine.inputs.get_boolean(input_name) else {
                    return false;
                };
                let rhs = match compare_to {
                    StringBool::Bool(value) => *value,
                    StringBool::String(reference) => {
                        match engine.inputs.get_boolean(reference.trim_start_matches('$')) {
                            Some(value) => value,
                            None => return false,
                        }
                    }
                };
                compare_eq(condition_type, lhs, rhs)
            }
        }
    }
}
