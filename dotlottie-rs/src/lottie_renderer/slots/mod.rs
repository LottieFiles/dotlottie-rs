mod color;
mod gradient;
mod image;
mod position;
mod scalar;
mod text;
mod vector;

pub use color::{ColorSlot, ColorValue};
pub use gradient::{GradientSlot, GradientStop};
pub use image::ImageSlot;
pub use position::PositionSlot;
pub use scalar::{ScalarSlot, ScalarValue};
pub use text::{TextCaps, TextDocument, TextJustify, TextKeyframe, TextSlot};
pub use vector::VectorSlot;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Bezier value can be either a single number or a vector for multi-dimensional properties
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BezierValue {
    Single(f32),
    Multi(Vec<f32>),
}

/// Bezier easing control point for keyframe interpolation
///
/// For linear interpolation:
/// - `o` (out): `{"x": [0, 0], "y": [0, 0]}`
/// - `i` (in): `{"x": [1, 1], "y": [1, 1]}`
///
/// The x axis represents time (0 = current keyframe, 1 = next keyframe)
/// The y axis represents value interpolation (0 = current value, 1 = next value)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bezier {
    pub x: BezierValue,
    pub y: BezierValue,
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

pub(crate) fn slots_to_json_string(
    slots: &BTreeMap<String, SlotType>,
) -> Result<String, serde_json::Error> {
    use serde_json::json;

    let mut lottie_slots = serde_json::Map::new();

    for (id, slot_type) in slots {
        lottie_slots.insert(id.clone(), json!({"p": slot_type}));
    }

    serde_json::to_string(&lottie_slots)
}

/// Serialize a single slot to JSON in ThorVG format: {"slot_id": {"p": value}}
pub(crate) fn single_slot_to_json_string(
    slot_id: &str,
    slot_type: &SlotType,
) -> Result<String, serde_json::Error> {
    use serde_json::json;

    let mut map = serde_json::Map::new();
    map.insert(slot_id.to_string(), json!({"p": slot_type}));
    serde_json::to_string(&map)
}

pub fn slot_to_json_string(slot: &SlotType) -> Result<String, serde_json::Error> {
    match slot {
        SlotType::Color(s) => serde_json::to_string(s),
        SlotType::Gradient(s) => serde_json::to_string(s),
        SlotType::Image(s) => serde_json::to_string(s),
        SlotType::Text(s) => serde_json::to_string(s),
        SlotType::Scalar(s) => serde_json::to_string(s),
        SlotType::Vector(s) => serde_json::to_string(s),
        SlotType::Position(s) => serde_json::to_string(s),
    }
}

pub fn slot_type_name(slot: &SlotType) -> &'static str {
    match slot {
        SlotType::Color(_) => "color",
        SlotType::Gradient(_) => "gradient",
        SlotType::Image(_) => "image",
        SlotType::Text(_) => "text",
        SlotType::Scalar(_) => "scalar",
        SlotType::Vector(_) => "vector",
        SlotType::Position(_) => "position",
    }
}

pub fn parse_slot_from_json(slot_type: &str, json: &str) -> Option<SlotType> {
    match slot_type {
        "color" => serde_json::from_str::<ColorSlot>(json).ok().map(SlotType::Color),
        "scalar" => serde_json::from_str::<ScalarSlot>(json).ok().map(SlotType::Scalar),
        "vector" => serde_json::from_str::<VectorSlot>(json).ok().map(SlotType::Vector),
        "position" => serde_json::from_str::<PositionSlot>(json).ok().map(SlotType::Position),
        "gradient" => serde_json::from_str::<GradientSlot>(json).ok().map(SlotType::Gradient),
        "image" => serde_json::from_str::<ImageSlot>(json).ok().map(SlotType::Image),
        "text" => serde_json::from_str::<TextSlot>(json).ok().map(SlotType::Text),
        _ => None,
    }
}

pub fn slots_from_json_string(
    json_str: &str,
) -> Result<BTreeMap<String, SlotType>, serde_json::Error> {
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

pub fn extract_slots_from_animation(animation_json: &str) -> BTreeMap<String, SlotType> {
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(animation_json);

    match parsed {
        Ok(json) => {
            // Prefer the top-level "slots" object if present (newer Lottie spec)
            if let Some(slots_obj) = json.get("slots") {
                if let Ok(slots_str) = serde_json::to_string(slots_obj) {
                    let slots = slots_from_json_string(&slots_str).unwrap_or_default();
                    if !slots.is_empty() {
                        return slots;
                    }
                }
            }

            // Fall back to walking the tree for "sid"-tagged properties
            let mut result = BTreeMap::new();
            collect_sid_slots(&json, &mut result);
            result
        }
        Err(_) => BTreeMap::new(),
    }
}

/// Walk the JSON tree and collect properties that have a "sid" (slot ID) tag.
/// These are animated properties with an explicit slot identifier, e.g.:
/// `{"a": 0, "k": [0.71, 0.192, 0.278], "sid": "ball_color"}`
fn collect_sid_slots(value: &serde_json::Value, result: &mut BTreeMap<String, SlotType>) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(sid)) = map.get("sid") {
                // This object is a slot-tagged property — parse it without the "sid" key
                let mut prop = map.clone();
                prop.remove("sid");
                let prop_value = serde_json::Value::Object(prop);
                if let Some(slot_type) = parse_slot_type(&prop_value) {
                    result.insert(sid.clone(), slot_type);
                }
            }
            // Continue walking children regardless
            for v in map.values() {
                collect_sid_slots(v, result);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_sid_slots(v, result);
            }
        }
        _ => {}
    }
}

fn parse_slot_type(value: &serde_json::Value) -> Option<SlotType> {
    if value.get("w").is_some() || value.get("h").is_some() || value.get("u").is_some() {
        serde_json::from_value::<ImageSlot>(value.clone())
            .ok()
            .map(SlotType::Image)
    } else if value.get("p").is_some() {
        serde_json::from_value::<GradientSlot>(value.clone())
            .ok()
            .map(SlotType::Gradient)
    } else if let Some(k) = value.get("k") {
        if let Some(arr) = k.as_array() {
            if arr.is_empty() {
                return None;
            }

            // Check if this is an array of keyframe objects (animated property)
            let first_element = &arr[0];
            let is_keyframe_array = first_element.is_object()
                && first_element.get("t").is_some()
                && first_element.get("s").is_some();

            if is_keyframe_array {
                if let Some(start_value) = first_element.get("s") {
                    if let Some(start_arr) = start_value.as_array() {
                        let len = start_arr.len();

                        if start_arr.iter().all(|v| v.is_number()) {
                            if len == 3 || len == 4 {
                                return serde_json::from_value::<ColorSlot>(value.clone())
                                    .ok()
                                    .map(SlotType::Color);
                            } else if len == 2 {
                                return serde_json::from_value::<VectorSlot>(value.clone())
                                    .ok()
                                    .map(SlotType::Vector);
                            } else if len == 1 {
                                return serde_json::from_value::<ScalarSlot>(value.clone())
                                    .ok()
                                    .map(SlotType::Scalar);
                            }
                        } else {
                            return serde_json::from_value::<TextSlot>(value.clone())
                                .ok()
                                .map(SlotType::Text);
                        }
                    } else if start_value.is_number() {
                        return serde_json::from_value::<ScalarSlot>(value.clone())
                            .ok()
                            .map(SlotType::Scalar);
                    } else if start_value.is_object() {
                        return serde_json::from_value::<TextSlot>(value.clone())
                            .ok()
                            .map(SlotType::Text);
                    }
                }
                return None;
            }

            let is_numeric_array = arr.iter().all(|v| v.is_number());

            if is_numeric_array {
                let len = arr.len();
                if len == 3 || len == 4 {
                    serde_json::from_value::<ColorSlot>(value.clone())
                        .ok()
                        .map(SlotType::Color)
                } else if len == 2 {
                    serde_json::from_value::<VectorSlot>(value.clone())
                        .ok()
                        .map(SlotType::Vector)
                } else if len == 1 {
                    serde_json::from_value::<ScalarSlot>(value.clone())
                        .ok()
                        .map(SlotType::Scalar)
                } else {
                    None
                }
            } else {
                // Non-numeric array, treat as text slot
                serde_json::from_value::<TextSlot>(value.clone())
                    .ok()
                    .map(SlotType::Text)
            }
        } else if k.is_number() {
            serde_json::from_value::<ScalarSlot>(value.clone())
                .ok()
                .map(SlotType::Scalar)
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

        if (arr.len() == 3 || arr.len() == 4) && arr.iter().all(|v| v.is_number()) {
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
