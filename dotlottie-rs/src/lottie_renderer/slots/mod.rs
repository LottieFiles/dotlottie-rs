mod color;
mod gradient;
mod image;
mod position;
mod scalar;
mod text;
mod vector;

pub use color::ColorSlot;
pub use gradient::{GradientSlot, GradientStop};
pub use image::ImageSlot;
pub use position::PositionSlot;
pub use scalar::ScalarSlot;
pub use text::{TextCaps, TextDocument, TextJustify, TextKeyframe, TextSlot};
pub use vector::VectorSlot;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bezier {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LottieKeyframe<T> {
    #[serde(rename = "t")]
    pub frame: u32,
    #[serde(rename = "s")]
    pub start_value: T,
    #[serde(skip_serializing_if = "Option::is_none", rename = "i")]
    pub in_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "o")]
    pub out_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "ti")]
    pub value_in_tangent: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "to")]
    pub value_out_tangent: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "h")]
    pub hold: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue<T> {
    Static(T),
    Animated(Vec<LottieKeyframe<T>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LottieProperty<T> {
    #[serde(rename = "a")]
    pub animated: u8,
    #[serde(rename = "k")]
    pub value: PropertyValue<T>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "x")]
    pub expression: Option<String>,
}

impl<T> LottieProperty<T> {
    pub fn static_value(value: T) -> Self {
        Self {
            animated: 0,
            value: PropertyValue::Static(value),
            expression: None,
        }
    }

    pub fn animated(keyframes: Vec<LottieKeyframe<T>>) -> Self {
        Self {
            animated: 1,
            value: PropertyValue::Animated(keyframes),
            expression: None,
        }
    }

    pub fn with_expression(mut self, expr: String) -> Self {
        self.expression = Some(expr);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SlotType {
    Color(ColorSlot),
    Gradient(GradientSlot),
    Image(ImageSlot),
    Text(TextSlot),
    Scalar(ScalarSlot),
    Vector(VectorSlot),
    Position(PositionSlot),
}

pub(crate) fn slots_to_json_string(slots: &BTreeMap<String, SlotType>) -> Result<String, serde_json::Error> {
    use serde_json::json;

    let mut lottie_slots = serde_json::Map::new();

    for (id, slot_type) in slots {
        lottie_slots.insert(id.clone(), json!({"p": slot_type}));
    }

    serde_json::to_string(&lottie_slots)
}

pub fn slots_from_json_string(json_str: &str) -> Result<BTreeMap<String, SlotType>, serde_json::Error> {
    use serde_json::Value;

    let slots_map: BTreeMap<String, Value> = serde_json::from_str(json_str)?;
    let mut result = BTreeMap::new();

    for (id, value) in slots_map {
        if let Some(p) = value.get("p") {
            if let Some(slot_type) = parse_slot_type(p) {
                result.insert(id, slot_type);
            }
        }
    }

    Ok(result)
}

fn parse_slot_type(value: &serde_json::Value) -> Option<SlotType> {
    if value.get("w").is_some() || value.get("h").is_some() || value.get("u").is_some() {
        serde_json::from_value::<ImageSlot>(value.clone()).ok().map(SlotType::Image)
    } else if value.get("p").is_some() {
        serde_json::from_value::<GradientSlot>(value.clone()).ok().map(SlotType::Gradient)
    } else if let Some(k) = value.get("k") {
        if k.is_array() {
            serde_json::from_value::<TextSlot>(value.clone()).ok().map(SlotType::Text)
        } else if let Some(k_obj) = k.as_object() {
            parse_animated_slot(k_obj)
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_animated_slot(k: &serde_json::Map<String, serde_json::Value>) -> Option<SlotType> {
    let k_value = k.get("k")?;

    if let Some(arr) = k_value.as_array() {
        if arr.is_empty() {
            return None;
        }

        if arr.len() == 3 && arr.iter().all(|v| v.is_number()) {
            serde_json::from_value::<ColorSlot>(serde_json::Value::Object(k.clone()))
                .ok()
                .map(SlotType::Color)
        } else if arr.len() == 2 && arr.iter().all(|v| v.is_number()) {
            // 2-element arrays can be either Vector or Position
            // Default to Vector during parsing (theme rules will explicitly create Position when needed)
            serde_json::from_value::<VectorSlot>(serde_json::Value::Object(k.clone()))
                .ok()
                .map(SlotType::Vector)
        } else if arr.len() == 1 && arr[0].is_number() {
            serde_json::from_value::<ScalarSlot>(serde_json::Value::Object(k.clone()))
                .ok()
                .map(SlotType::Scalar)
        } else {
            None
        }
    } else {
        None
    }
}
