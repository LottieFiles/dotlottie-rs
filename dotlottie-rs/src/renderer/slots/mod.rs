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

use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Streams `{"p": <slot_value>}` without intermediate `Value` allocation.
struct SlotWrapper<'a>(&'a SlotType);

impl<'a> serde::Serialize for SlotWrapper<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("p", self.0)?;
        map.end()
    }
}

/// Streams the entire `{"id":{"p":val},...}` object in a single pass —
/// no intermediate `serde_json::Map` or `Value` tree.
struct SlotsMapWrapper<'a>(&'a BTreeMap<String, SlotType>);

impl<'a> serde::Serialize for SlotsMapWrapper<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (id, slot_type) in self.0 {
            map.serialize_entry(id.as_str(), &SlotWrapper(slot_type))?;
        }
        map.end()
    }
}

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
    serde_json::to_string(&SlotsMapWrapper(slots))
}

/// Write all slots as batch JSON directly to a byte buffer, avoiding String allocation.
/// The buffer is cleared before writing. A null terminator is NOT appended (caller handles that).
pub(crate) fn slots_to_json_writer(
    slots: &BTreeMap<String, SlotType>,
    buf: &mut Vec<u8>,
) -> Result<(), serde_json::Error> {
    buf.clear();
    serde_json::to_writer(buf as &mut Vec<u8>, &SlotsMapWrapper(slots))
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
        "color" => serde_json::from_str::<ColorSlot>(json)
            .ok()
            .map(SlotType::Color),
        "scalar" => serde_json::from_str::<ScalarSlot>(json)
            .ok()
            .map(SlotType::Scalar),
        "vector" => serde_json::from_str::<VectorSlot>(json)
            .ok()
            .map(SlotType::Vector),
        "position" => serde_json::from_str::<PositionSlot>(json)
            .ok()
            .map(SlotType::Position),
        "gradient" => serde_json::from_str::<GradientSlot>(json)
            .ok()
            .map(SlotType::Gradient),
        "image" => serde_json::from_str::<ImageSlot>(json)
            .ok()
            .map(SlotType::Image),
        "text" => serde_json::from_str::<TextSlot>(json)
            .ok()
            .map(SlotType::Text),
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
            // Prefer the top-level "slots" object if present (newer Lottie spec).
            // Work directly with the Value to avoid a serialize→deserialize round-trip.
            if let Some(serde_json::Value::Object(slots_map)) = json.get("slots") {
                let mut result = BTreeMap::new();
                for (id, value) in slots_map {
                    if let Some(p) = value.get("p") {
                        if let Some(slot_type) = parse_slot_type(p) {
                            result.insert(id.clone(), slot_type);
                        }
                    }
                }
                if !result.is_empty() {
                    return result;
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;

    // ── Group A: extract_slots_from_animation ─────────────────────

    #[test]
    fn extract_top_level_color_slot() {
        let anim = r#"{"v":"5.0","slots":{"my_color":{"p":{"a":0,"k":[1.0,0.0,0.0]}}}}"#;
        let slots = extract_slots_from_animation(anim);
        assert_eq!(slots.len(), 1);
        assert_eq!(slot_type_name(slots.get("my_color").unwrap()), "color");
    }

    #[test]
    fn extract_prefers_top_level_over_sid() {
        let anim = json!({
            "slots": {
                "from_slots": {"p": {"a": 0, "k": [1.0, 0.0, 0.0]}}
            },
            "layers": [{
                "ks": {
                    "o": {"a": 0, "k": [0.5, 0.5, 0.5], "sid": "from_sid"}
                }
            }]
        })
        .to_string();
        let slots = extract_slots_from_animation(&anim);
        assert_eq!(slots.len(), 1);
        assert!(slots.contains_key("from_slots"));
        assert!(!slots.contains_key("from_sid"));
    }

    #[test]
    fn extract_falls_back_to_sid_when_slots_empty() {
        let anim = json!({
            "slots": {},
            "layers": [{
                "ks": {
                    "o": {"a": 0, "k": [0.5, 0.5, 0.5], "sid": "my_color"}
                }
            }]
        })
        .to_string();
        let slots = extract_slots_from_animation(&anim);
        assert_eq!(slots.len(), 1);
        assert!(slots.contains_key("my_color"));
    }

    #[test]
    fn extract_no_slots_key_uses_sid() {
        let anim = json!({
            "layers": [{
                "ks": {
                    "o": {"a": 0, "k": [0.5, 0.5, 0.5], "sid": "my_color"}
                }
            }]
        })
        .to_string();
        let slots = extract_slots_from_animation(&anim);
        assert_eq!(slots.len(), 1);
        assert!(slots.contains_key("my_color"));
    }

    #[test]
    fn extract_invalid_json_returns_empty() {
        let slots = extract_slots_from_animation("not valid json {{");
        assert!(slots.is_empty());
    }

    #[test]
    fn extract_empty_string_returns_empty() {
        let slots = extract_slots_from_animation("");
        assert!(slots.is_empty());
    }

    #[test]
    fn extract_animation_with_no_slots() {
        let anim = json!({"v": "5.0", "fr": 30, "w": 100, "h": 100, "layers": []}).to_string();
        let slots = extract_slots_from_animation(&anim);
        assert!(slots.is_empty());
    }

    // ── Group B: parse_slot_type static branches ──────────────────

    #[test]
    fn parse_static_color_4_elem() {
        let val = json!({"a": 0, "k": [1.0, 0.0, 0.0, 1.0]});
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "color");
    }

    #[test]
    fn parse_static_vector_2_elem() {
        let val = json!({"a": 0, "k": [100.0, 200.0]});
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "vector");
    }

    #[test]
    fn parse_static_scalar_single_elem_array() {
        let val = json!({"a": 0, "k": [42.0]});
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "scalar");
    }

    #[test]
    fn parse_image_slot() {
        let val = json!({"w": 100, "h": 100, "u": "images/", "p": "img.png"});
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "image");
    }

    #[test]
    fn parse_gradient_slot() {
        let val = json!({"p": 2, "k": {"a": 0, "k": [0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]}});
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "gradient");
    }

    #[test]
    fn parse_empty_k_array_returns_none() {
        let val = json!({"a": 0, "k": []});
        assert!(parse_slot_type(&val).is_none());
    }

    #[test]
    fn parse_no_k_no_w_no_p_returns_none() {
        let val = json!({"a": 0, "x": "something"});
        assert!(parse_slot_type(&val).is_none());
    }

    #[test]
    fn parse_k_is_string_returns_none() {
        let val = json!({"k": "invalid"});
        assert!(parse_slot_type(&val).is_none());
    }

    #[test]
    fn parse_5_elem_array_returns_none() {
        let val = json!({"k": [1.0, 2.0, 3.0, 4.0, 5.0]});
        assert!(parse_slot_type(&val).is_none());
    }

    // ── Group C: parse_slot_type animated branches ────────────────

    #[test]
    fn parse_animated_color_keyframes() {
        let val = json!({"a": 1, "k": [
            {"t": 0, "s": [1.0, 0.0, 0.0]},
            {"t": 30, "s": [0.0, 1.0, 0.0]}
        ]});
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "color");
    }

    #[test]
    fn parse_animated_scalar_keyframes_number() {
        let val = json!({"a": 1, "k": [
            {"t": 0, "s": 50.0},
            {"t": 30, "s": 100.0}
        ]});
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "scalar");
    }

    #[test]
    fn parse_animated_scalar_keyframes_array() {
        let val = json!({"a": 1, "k": [
            {"t": 0, "s": [50.0]},
            {"t": 30, "s": [100.0]}
        ]});
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "scalar");
    }

    #[test]
    fn parse_animated_text_keyframes_object() {
        let val = json!({"k": [
            {"t": 0, "s": {"t": "Hello"}},
            {"t": 30, "s": {"t": "World"}}
        ]});
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "text");
    }

    #[test]
    fn parse_animated_text_keyframes_non_numeric() {
        // When "s" is a non-numeric array, detection tries TextSlot but
        // deserialization fails because TextKeyframe expects "s" to be
        // a TextDocument object, not an array.
        let val = json!({"k": [{"t": 0, "s": ["not_a_number"]}]});
        assert!(parse_slot_type(&val).is_none());
    }

    // ── Group D: parse_animated_slot (nested k.k pattern) ─────────

    #[test]
    fn parse_nested_animated_color() {
        let val = json!({"k": {"a": 0, "k": [1.0, 0.0, 0.0]}});
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "color");
    }

    #[test]
    fn parse_nested_animated_vector() {
        let val = json!({"k": {"a": 0, "k": [100.0, 200.0]}});
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "vector");
    }

    #[test]
    fn parse_nested_animated_scalar() {
        let val = json!({"k": {"a": 0, "k": [42.0]}});
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "scalar");
    }

    #[test]
    fn parse_nested_animated_empty_k() {
        let val = json!({"k": {"a": 0, "k": []}});
        assert!(parse_slot_type(&val).is_none());
    }

    // ── Group E: Serialization round-trips ────────────────────────

    #[test]
    fn round_trip_color() {
        let mut map = BTreeMap::new();
        map.insert(
            "c".to_string(),
            SlotType::Color(ColorSlot::new([1.0, 0.0, 0.0])),
        );
        let json = slots_to_json_string(&map).unwrap();
        let parsed = slots_from_json_string(&json).unwrap();
        let json2 = slots_to_json_string(&parsed).unwrap();
        assert_eq!(json, json2);
    }

    #[test]
    fn round_trip_scalar() {
        let mut map = BTreeMap::new();
        map.insert("s".to_string(), SlotType::Scalar(ScalarSlot::new(42.0)));
        let json = slots_to_json_string(&map).unwrap();
        let parsed = slots_from_json_string(&json).unwrap();
        let json2 = slots_to_json_string(&parsed).unwrap();
        assert_eq!(json, json2);
    }

    #[test]
    fn round_trip_vector() {
        let mut map = BTreeMap::new();
        map.insert(
            "v".to_string(),
            SlotType::Vector(VectorSlot::static_value([100.0, 200.0])),
        );
        let json = slots_to_json_string(&map).unwrap();
        let parsed = slots_from_json_string(&json).unwrap();
        let json2 = slots_to_json_string(&parsed).unwrap();
        assert_eq!(json, json2);
    }

    #[test]
    fn round_trip_image() {
        let img = ImageSlot {
            width: Some(100),
            height: Some(200),
            directory: None,
            path: Some("img.png".to_string()),
            embed: None,
        };
        let mut map = BTreeMap::new();
        map.insert("img".to_string(), SlotType::Image(img));
        let json = slots_to_json_string(&map).unwrap();
        let parsed = slots_from_json_string(&json).unwrap();
        let json2 = slots_to_json_string(&parsed).unwrap();
        assert_eq!(json, json2);
    }

    #[test]
    fn round_trip_text() {
        let mut map = BTreeMap::new();
        map.insert("txt".to_string(), SlotType::Text(TextSlot::new("Hello")));
        let json = slots_to_json_string(&map).unwrap();
        let parsed = slots_from_json_string(&json).unwrap();
        let json2 = slots_to_json_string(&parsed).unwrap();
        assert_eq!(json, json2);
    }

    #[test]
    fn round_trip_gradient() {
        let grad = GradientSlot::new(vec![
            GradientStop {
                offset: 0.0,
                color: [1.0, 0.0, 0.0, 1.0],
            },
            GradientStop {
                offset: 1.0,
                color: [0.0, 0.0, 1.0, 1.0],
            },
        ]);
        let mut map = BTreeMap::new();
        map.insert("grad".to_string(), SlotType::Gradient(grad));
        let json = slots_to_json_string(&map).unwrap();
        let parsed = slots_from_json_string(&json).unwrap();
        let json2 = slots_to_json_string(&parsed).unwrap();
        assert_eq!(json, json2);
    }

    #[test]
    fn round_trip_empty_map() {
        let map: BTreeMap<String, SlotType> = BTreeMap::new();
        let json = slots_to_json_string(&map).unwrap();
        assert_eq!(json, "{}");
        let parsed = slots_from_json_string(&json).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn round_trip_multiple_types() {
        let mut map = BTreeMap::new();
        map.insert(
            "color".to_string(),
            SlotType::Color(ColorSlot::new([1.0, 0.0, 0.0])),
        );
        map.insert(
            "scalar".to_string(),
            SlotType::Scalar(ScalarSlot::new(42.0)),
        );
        map.insert(
            "vector".to_string(),
            SlotType::Vector(VectorSlot::static_value([10.0, 20.0])),
        );
        let json = slots_to_json_string(&map).unwrap();
        let parsed = slots_from_json_string(&json).unwrap();
        assert_eq!(parsed.len(), 3);
        let json2 = slots_to_json_string(&parsed).unwrap();
        assert_eq!(json, json2);
    }

    // ── Group F: slots_to_json_writer ─────────────────────────────

    #[test]
    fn json_writer_matches_json_string() {
        let mut map = BTreeMap::new();
        map.insert(
            "c".to_string(),
            SlotType::Color(ColorSlot::new([1.0, 0.0, 0.0])),
        );
        let json_str = slots_to_json_string(&map).unwrap();
        let mut buf = Vec::new();
        slots_to_json_writer(&map, &mut buf).unwrap();
        assert_eq!(json_str.as_bytes(), &buf[..]);
    }

    #[test]
    fn json_writer_clears_buffer() {
        let mut buf = vec![1, 2, 3, 4, 5];
        let map: BTreeMap<String, SlotType> = BTreeMap::new();
        slots_to_json_writer(&map, &mut buf).unwrap();
        assert_eq!(buf, b"{}");
    }

    // ── Group G: slots_from_json_string error handling ────────────

    #[test]
    fn from_json_string_invalid_json() {
        assert!(slots_from_json_string("not json").is_err());
    }

    #[test]
    fn from_json_string_empty_object() {
        let result = slots_from_json_string("{}").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn from_json_string_missing_p_key() {
        // Entry without "p" wrapper is silently skipped
        let result = slots_from_json_string(r#"{"foo":{"k":[1,0,0]}}"#).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn from_json_string_unparseable_slot_skipped() {
        // One good slot with proper "p" wrapper, one bad ("p" is a string)
        let json = r#"{"good":{"p":{"a":0,"k":[1.0,0.0,0.0]}},"bad":{"p":"nope"}}"#;
        let result = slots_from_json_string(json).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("good"));
    }

    // ── Group H: parse_slot_from_json all branches ────────────────

    #[test]
    fn parse_from_json_scalar() {
        let slot = parse_slot_from_json("scalar", r#"{"a":0,"k":42.0}"#).unwrap();
        assert_eq!(slot_type_name(&slot), "scalar");
    }

    #[test]
    fn parse_from_json_vector() {
        let slot = parse_slot_from_json("vector", r#"{"a":0,"k":[1.0,2.0]}"#).unwrap();
        assert_eq!(slot_type_name(&slot), "vector");
    }

    #[test]
    fn parse_from_json_position() {
        let slot = parse_slot_from_json("position", r#"{"a":0,"k":[1.0,2.0]}"#).unwrap();
        assert_eq!(slot_type_name(&slot), "position");
    }

    #[test]
    fn parse_from_json_gradient() {
        let json = r#"{"k":{"a":0,"k":[0.0,1.0,0.0,0.0]},"p":1}"#;
        let slot = parse_slot_from_json("gradient", json).unwrap();
        assert_eq!(slot_type_name(&slot), "gradient");
    }

    #[test]
    fn parse_from_json_image() {
        let slot = parse_slot_from_json("image", r#"{"w":100,"h":100}"#).unwrap();
        assert_eq!(slot_type_name(&slot), "image");
    }

    #[test]
    fn parse_from_json_text() {
        let json = r#"{"k":[{"t":0,"s":{"t":"Hello"}}]}"#;
        let slot = parse_slot_from_json("text", json).unwrap();
        assert_eq!(slot_type_name(&slot), "text");
    }

    #[test]
    fn parse_from_json_unknown_type() {
        assert!(parse_slot_from_json("unknown", r#"{"a":0,"k":[1.0]}"#).is_none());
    }

    #[test]
    fn parse_from_json_malformed() {
        assert!(parse_slot_from_json("color", "not json").is_none());
    }
}
