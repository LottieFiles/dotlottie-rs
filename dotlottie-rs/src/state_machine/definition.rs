use crate::json::{array_of, opt, Value};
use crate::string::{DotString, DotStringInterner};

use super::{inputs::Input, interactions::Interaction, states::State, GLOBAL_INPUT_PREFIX};

#[derive(Debug, Clone, PartialEq)]
pub enum StringNumberBool {
    String(String),
    F32(f32),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringBool {
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringNumber {
    String(String),
    F32(f32),
}

#[derive(Debug, Default)]
pub struct StateMachine {
    pub initial: DotString,
    pub states: Vec<State>,
    pub interactions: Option<Vec<Interaction>>,
    pub inputs: Option<Vec<Input>>,
}

impl StateMachine {
    /// Canonicalize every identifier through the shared interner so runtime
    /// comparisons hit the `Arc::ptr_eq` fast path.
    pub(crate) fn intern_identifiers(&mut self, interner: &mut DotStringInterner) {
        self.initial = interner.intern(self.initial.as_str());
        for state in &mut self.states {
            state.intern_identifiers(interner);
        }
        if let Some(interactions) = &mut self.interactions {
            for i in interactions {
                i.intern_identifiers(interner);
            }
        }
    }
}

pub(crate) fn dot_string(v: &Value) -> Option<DotString> {
    DotString::try_new(v.as_str()?).ok()
}

pub(crate) fn string_number(v: &Value) -> Option<StringNumber> {
    if let Some(s) = v.as_str() {
        return Some(StringNumber::String(s.to_owned()));
    }
    v.as_f32().map(StringNumber::F32)
}

pub(crate) fn string_bool(v: &Value) -> Option<StringBool> {
    if let Some(s) = v.as_str() {
        return Some(StringBool::String(s.to_owned()));
    }
    v.as_bool().map(StringBool::Bool)
}

pub(crate) fn string_number_bool(v: &Value) -> Option<StringNumberBool> {
    if let Some(s) = v.as_str() {
        return Some(StringNumberBool::String(s.to_owned()));
    }
    if let Some(n) = v.as_f32() {
        return Some(StringNumberBool::F32(n));
    }
    v.as_bool().map(StringNumberBool::Bool)
}

pub fn state_machine_parse(json: &str) -> Result<StateMachine, super::Error> {
    let root = Value::parse(json).map_err(|e| super::Error::ParsingError(e.to_string()))?;
    state_machine_from_value(&root)
        .ok_or_else(|| super::Error::ParsingError("invalid state machine definition".to_string()))
}

fn state_machine_from_value(root: &Value) -> Option<StateMachine> {
    let initial = dot_string(root.get("initial")?)?;
    let states = array_of(root.get("states")?, super::states::state_from_json)?;
    let interactions = opt(root.get("interactions"), |v| {
        array_of(v, super::interactions::interaction_from_json)
    })?;
    // Drop @-prefixed user declarations. The @ namespace is reserved for built-ins
    let inputs = opt(root.get("inputs"), |v| {
        array_of(v, super::inputs::input_from_json)
    })?
    .map(|mut v| {
        v.retain(|input| {
            let name = match input {
                Input::Numeric { name, .. }
                | Input::String { name, .. }
                | Input::Boolean { name, .. }
                | Input::Event { name } => name,
            };
            !name.starts_with(GLOBAL_INPUT_PREFIX)
        });
        v
    });
    Some(StateMachine {
        initial,
        states,
        interactions,
        inputs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::actions::Action;

    const SM: &str = r#"{
        "initial": "a",
        "states": [
            {"type": "PlaybackState", "name": "a", "animation": "anim1", "loop": true,
             "loopCount": 3, "final": false, "autoplay": true, "mode": "Reverse",
             "speed": 2, "segment": "intro", "backgroundColor": 4278190335,
             "transitions": [
                {"type": "Tweened", "toState": "b", "duration": 0.5, "easing": [0.4, 0, 0.2, 1],
                 "guards": [{"type": "Numeric", "inputName": "n", "conditionType": "GreaterThan", "compareTo": 5}]}
             ],
             "entryActions": [{"type": "Increment", "inputName": "n", "value": 1}, {"type": "SetRandom", "inputName": "n"}]},
            {"type": "GlobalState", "name": "b", "transitions": [
                {"type": "Transition", "toState": "a",
                 "guards": [{"type": "Event", "inputName": "go"}]}
            ]}
        ],
        "interactions": [
            {"type": "Click", "layerName": "btn", "actions": [{"type": "Fire", "inputName": "go"}]},
            {"type": "OnComplete", "stateName": "a", "actions": [{"type": "Toggle", "inputName": "flag"}]}
        ],
        "inputs": [
            {"type": "Numeric", "name": "n", "value": 0},
            {"type": "Boolean", "name": "flag", "value": false},
            {"type": "Numeric", "name": "@reserved", "value": 1},
            {"type": "Event", "name": "go"}
        ]
    }"#;

    #[test]
    fn parses_full_state_machine() {
        let sm = state_machine_parse(SM).expect("parses");
        assert_eq!(sm.initial, "a");
        assert_eq!(sm.states.len(), 2);
        let s0 = &sm.states[0];
        assert_eq!(s0.name(), "a");
        assert_eq!(s0.animation(), "anim1");
        let t = &s0.transitions()[0];
        assert_eq!(t.to_state, "b");
        assert_eq!(t.tween.duration, 500.0); // 0.5s → ms
        assert_eq!(t.tween.easing, [0.4, 0.0, 0.2, 1.0]);
        let State::PlaybackState { entry_actions, .. } = s0 else {
            panic!("expected playback state");
        };
        assert!(matches!(
            entry_actions.as_ref().unwrap()[1],
            Action::SetRandom { ref min, ref max, ref integer, .. }
                if min.is_none() && max.is_none() && integer.is_none()
        ));
        assert_eq!(sm.interactions.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn at_prefixed_inputs_are_filtered() {
        let sm = state_machine_parse(SM).unwrap();
        let inputs = sm.inputs.as_ref().unwrap();
        assert_eq!(inputs.len(), 3);
        assert!(!inputs.iter().any(|i| matches!(
            i,
            crate::state_machine::inputs::Input::Numeric { name, .. } if name == "@reserved"
        )));
    }

    #[test]
    fn untagged_compare_to_coerces() {
        let sm = state_machine_parse(SM).unwrap();
        let g = &sm.states[0].transitions()[0].guards()[0];
        assert!(matches!(
            g,
            crate::state_machine::transitions::guard::Guard::Numeric {
                compare_to: StringNumberBool::F32(v),
                ..
            } if *v == 5.0
        ));
    }

    #[test]
    fn instant_and_zero_duration_tween_are_the_same_thing() {
        let sm = state_machine_parse(
            r#"{"initial":"a","states":[{"type":"GlobalState","name":"a","transitions":[
                {"type":"Transition","toState":"a"},
                {"type":"Tweened","toState":"a","duration":0,"easing":[0,0,1,1]}]}]}"#,
        )
        .unwrap();
        let transitions = sm.states[0].transitions();
        assert!(transitions[0].tween.is_instant());
        assert!(transitions[1].tween.is_instant());
    }

    #[test]
    fn increment_without_value_defaults_to_one() {
        let sm = state_machine_parse(
            r#"{"initial":"a","states":[{"type":"GlobalState","name":"a","transitions":[],
                "entryActions":[{"type":"Increment","inputName":"n"},
                                {"type":"Decrement","inputName":"n"}]}]}"#,
        )
        .unwrap();
        let State::GlobalState { entry_actions, .. } = &sm.states[0] else {
            panic!("expected global state");
        };
        for action in entry_actions.as_ref().unwrap() {
            assert!(matches!(
                action,
                Action::Increment { value: StringNumber::F32(v), .. }
                | Action::Decrement { value: StringNumber::F32(v), .. } if *v == 1.0
            ));
        }
    }

    #[test]
    fn rejects_bad_definitions() {
        assert!(state_machine_parse("not json").is_err());
        assert!(state_machine_parse(r#"{"states":[]}"#).is_err()); // missing initial
        assert!(
            state_machine_parse(r#"{"initial":"a","states":[{"type":"Nope","name":"x"}]}"#)
                .is_err()
        );
        // Tweened requires easing with exactly 4 numbers
        assert!(state_machine_parse(
            r#"{"initial":"a","states":[{"type":"GlobalState","name":"a","transitions":[
                {"type":"Tweened","toState":"a","duration":1,"easing":[0,0,1]}]}]}"#
        )
        .is_err());
    }
}
