//! State-declared slots (prototype).
//!
//! A `PlaybackState` may declare a scoped styling overlay of interpolable
//! slot values (`docs/spec-updates/state-slots.md`). Declared values apply
//! instantly on plain transitions, interpolate on `Tweened` transitions
//! (riding the transition's clock), release to their base values on state
//! exit, and are live-bound to referenced Numeric inputs while the state is
//! current.

use serde::Deserialize;

use crate::lottie_renderer::{ColorSlot, PositionSlot, ScalarSlot, SlotType, VectorSlot};
use crate::string::{DotString, DotStringInterner};

use super::state_machine::StringNumber;
use super::{StateMachineEngine, GLOBAL_INPUT_PREFIX};

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum StateSlotType {
    Color,
    Scalar,
    Vector,
    Position,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum StateSlotValue {
    Single(StringNumber),
    Multi(Vec<StringNumber>),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StateSlot {
    pub slot_id: DotString,
    #[serde(rename = "type")]
    pub slot_type: StateSlotType,
    pub value: StateSlotValue,
}

/// An in-flight interpolation of one slot during a `Tweened` transition.
#[derive(Debug, Clone)]
pub(crate) struct SlotLerp {
    pub slot_id: String,
    pub slot_type: StateSlotType,
    pub from: Vec<f32>,
    pub to: Vec<f32>,
}

impl StateSlot {
    pub(crate) fn intern_identifiers(&mut self, interner: &mut DotStringInterner) {
        self.slot_id = interner.intern(self.slot_id.as_str());
    }

    fn elements(&self) -> &[StringNumber] {
        match &self.value {
            StateSlotValue::Single(v) => std::slice::from_ref(v),
            StateSlotValue::Multi(v) => v,
        }
    }

    /// The Numeric input names this declaration references (`$name` elements).
    pub(crate) fn referenced_inputs(&self) -> impl Iterator<Item = &str> {
        self.elements().iter().filter_map(|e| match e {
            StringNumber::String(s) => s.strip_prefix('$'),
            StringNumber::F32(_) => None,
        })
    }

    /// Resolve the declared value to numeric components, or `None` if any
    /// reference is unresolvable or the arity doesn't match the slot type.
    /// `@` globals are rejected: bindings re-fire on input writes, and
    /// globals tick continuously (see D6 in the spec draft).
    pub(crate) fn resolve(&self, engine: &StateMachineEngine) -> Option<Vec<f32>> {
        let elements = self.elements();

        let expected = match self.slot_type {
            StateSlotType::Scalar => 1..=1,
            StateSlotType::Vector | StateSlotType::Position => 2..=2,
            StateSlotType::Color => 3..=4,
        };
        if !expected.contains(&elements.len()) {
            return None;
        }

        let mut comps = Vec::with_capacity(elements.len());
        for element in elements {
            let v = match element {
                StringNumber::F32(v) => *v,
                StringNumber::String(s) => {
                    let name = s.strip_prefix('$')?;
                    if name.starts_with(GLOBAL_INPUT_PREFIX) {
                        return None;
                    }
                    engine.get_numeric_input(name)?
                }
            };
            comps.push(v);
        }

        // The renderer's color value is RGB-only; a declared alpha is dropped
        // so interpolation endpoints always have matching arity.
        if self.slot_type == StateSlotType::Color {
            comps.truncate(3);
        }

        Some(comps)
    }
}

/// Build a static slot value of the declared type from resolved components.
/// Color components are clamped to [0, 1] (easing curves can overshoot).
pub(crate) fn build_slot(slot_type: StateSlotType, comps: &[f32]) -> Option<SlotType> {
    match slot_type {
        StateSlotType::Color if comps.len() >= 3 => Some(SlotType::Color(ColorSlot::new([
            comps[0].clamp(0.0, 1.0),
            comps[1].clamp(0.0, 1.0),
            comps[2].clamp(0.0, 1.0),
        ]))),
        StateSlotType::Scalar if comps.len() == 1 => {
            Some(SlotType::Scalar(ScalarSlot::new(comps[0])))
        }
        StateSlotType::Vector if comps.len() == 2 => Some(SlotType::Vector(
            VectorSlot::static_value([comps[0], comps[1]]),
        )),
        StateSlotType::Position if comps.len() == 2 => Some(SlotType::Position(
            PositionSlot::static_value([comps[0], comps[1]]),
        )),
        _ => None,
    }
}

/// Extract numeric components from a slot value, if it is a static value of
/// an interpolable type. Animated (keyframed) values and non-numeric slot
/// types return `None` — those can't serve as interpolation endpoints.
pub(crate) fn static_components(slot: &SlotType) -> Option<Vec<f32>> {
    use crate::lottie_renderer::slots::PropertyValue;

    match slot {
        SlotType::Color(p) => match &p.value {
            PropertyValue::Static(c) => Some(c.0.to_vec()),
            PropertyValue::Animated(_) => None,
        },
        SlotType::Scalar(p) => match &p.value {
            PropertyValue::Static(s) => Some(vec![s.0]),
            PropertyValue::Animated(_) => None,
        },
        SlotType::Vector(p) | SlotType::Position(p) => match &p.value {
            PropertyValue::Static(v) => Some(v.to_vec()),
            PropertyValue::Animated(_) => None,
        },
        SlotType::Gradient(_) | SlotType::Image(_) | SlotType::Text(_) => None,
    }
}

/// Whether a declared slot type is compatible with the slot's current value.
/// Vector and Position are interchangeable — they are type-identical
/// (`LottieProperty<[f32; 2]>`) and authored-slot extraction defaults
/// 2-element values to Vector.
pub(crate) fn type_compatible(declared: StateSlotType, current: &SlotType) -> bool {
    matches!(
        (declared, current),
        (StateSlotType::Color, SlotType::Color(_))
            | (StateSlotType::Scalar, SlotType::Scalar(_))
            | (
                StateSlotType::Vector | StateSlotType::Position,
                SlotType::Vector(_) | SlotType::Position(_)
            )
    )
}

pub(crate) fn lerp_components(from: &[f32], to: &[f32], progress: f32) -> Vec<f32> {
    from.iter()
        .zip(to.iter())
        .map(|(f, t)| f + (t - f) * progress)
        .collect()
}
