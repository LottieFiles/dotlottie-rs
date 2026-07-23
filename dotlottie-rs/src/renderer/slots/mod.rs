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

use gradient::{gradient_slot_from_json, write_gradient_slot};
use image::{image_slot_from_json, write_image_slot};
use std::collections::BTreeMap;
use text::{text_slot_from_json, write_text_slot};

/// Bezier value can be either a single number or a vector for multi-dimensional properties
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct Bezier {
    pub x: BezierValue,
    pub y: BezierValue,
}

#[derive(Debug, Clone)]
pub struct LottieKeyframe<T> {
    pub frame: u32,
    pub start_value: T,
    pub in_tangent: Option<Bezier>,
    pub out_tangent: Option<Bezier>,
    pub value_in_tangent: Option<Vec<f32>>,
    pub value_out_tangent: Option<Vec<f32>>,
    pub hold: Option<u8>,
}

#[derive(Debug, Clone)]
pub enum PropertyValue<T> {
    Static(T),
    Animated(Vec<LottieKeyframe<T>>),
}

#[derive(Debug, Clone)]
pub struct LottieProperty<T> {
    pub animated: u8,
    pub value: PropertyValue<T>,
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

use crate::json::{array_of, f32_vec, opt, write_f32, write_seq, write_str, ObjWriter, Value};
use std::fmt::Write as _;

pub(crate) trait SlotValue: Sized {
    fn from_json(v: &Value) -> Option<Self>;
    fn write(&self, out: &mut String);
}

impl SlotValue for [f32; 2] {
    fn from_json(v: &Value) -> Option<Self> {
        crate::json::f32_array(v)
    }
    fn write(&self, out: &mut String) {
        write_f32_slice(self, out);
    }
}

impl SlotValue for Vec<f32> {
    fn from_json(v: &Value) -> Option<Self> {
        f32_vec(v)
    }
    fn write(&self, out: &mut String) {
        write_f32_slice(self, out);
    }
}

pub(crate) fn write_f32_slice(vals: &[f32], out: &mut String) {
    write_seq(out, vals, |v, out| write_f32(*v, out));
}

fn bezier_value_from_json(v: &Value) -> Option<BezierValue> {
    if let Some(n) = v.as_f32() {
        Some(BezierValue::Single(n))
    } else {
        f32_vec(v).map(BezierValue::Multi)
    }
}

pub(crate) fn bezier_from_json(v: &Value) -> Option<Bezier> {
    Some(Bezier {
        x: bezier_value_from_json(v.get("x")?)?,
        y: bezier_value_from_json(v.get("y")?)?,
    })
}

fn write_bezier_value(b: &BezierValue, out: &mut String) {
    match b {
        BezierValue::Single(v) => write_f32(*v, out),
        BezierValue::Multi(vs) => write_f32_slice(vs, out),
    }
}

pub(crate) fn write_bezier(b: &Bezier, out: &mut String) {
    let mut o = ObjWriter::new(out);
    write_bezier_value(&b.x, o.field("x"));
    write_bezier_value(&b.y, o.field("y"));
    o.finish();
}

fn keyframe_from_json<T: SlotValue>(v: &Value) -> Option<LottieKeyframe<T>> {
    Some(LottieKeyframe {
        frame: v.u32_field("t")?,
        start_value: T::from_json(v.get("s")?)?,
        in_tangent: opt(v.get("i"), bezier_from_json)?,
        out_tangent: opt(v.get("o"), bezier_from_json)?,
        value_in_tangent: opt(v.get("ti"), f32_vec)?,
        value_out_tangent: opt(v.get("to"), f32_vec)?,
        hold: opt(v.get("h"), Value::as_u8)?,
    })
}

fn write_keyframe<T: SlotValue>(kf: &LottieKeyframe<T>, out: &mut String) {
    let mut o = ObjWriter::new(out);
    let _ = write!(o.field("t"), "{}", kf.frame);
    kf.start_value.write(o.field("s"));
    if let Some(b) = &kf.in_tangent {
        write_bezier(b, o.field("i"));
    }
    if let Some(b) = &kf.out_tangent {
        write_bezier(b, o.field("o"));
    }
    if let Some(v) = &kf.value_in_tangent {
        write_f32_slice(v, o.field("ti"));
    }
    if let Some(v) = &kf.value_out_tangent {
        write_f32_slice(v, o.field("to"));
    }
    if let Some(h) = kf.hold {
        let _ = write!(o.field("h"), "{h}");
    }
    o.finish();
}

pub(crate) fn property_from_json<T: SlotValue>(v: &Value) -> Option<LottieProperty<T>> {
    let animated = v.u8_field("a")?;
    let k = v.get("k")?;
    let value = if let Some(s) = T::from_json(k) {
        PropertyValue::Static(s)
    } else {
        PropertyValue::Animated(array_of(k, keyframe_from_json::<T>)?)
    };
    let expression = v.opt_str_field("x")?;
    Some(LottieProperty {
        animated,
        value,
        expression,
    })
}

pub(crate) fn write_property<T: SlotValue>(p: &LottieProperty<T>, out: &mut String) {
    let mut o = ObjWriter::new(out);
    let _ = write!(o.field("a"), "{}", p.animated);
    {
        let out = o.field("k");
        match &p.value {
            PropertyValue::Static(v) => v.write(out),
            PropertyValue::Animated(kfs) => write_seq(out, kfs, write_keyframe),
        }
    }
    if let Some(x) = &p.expression {
        write_str(x, o.field("x"));
    }
    o.finish();
}

#[derive(Debug, Clone)]
pub enum SlotType {
    Color(ColorSlot),
    Gradient(GradientSlot),
    Image(ImageSlot),
    Text(TextSlot),
    Scalar(ScalarSlot),
    Vector(VectorSlot),
    Position(PositionSlot),
}

fn write_slot(slot: &SlotType, out: &mut String) {
    match slot {
        SlotType::Color(s) => write_property(s, out),
        SlotType::Gradient(s) => write_gradient_slot(s, out),
        SlotType::Image(s) => write_image_slot(s, out),
        SlotType::Text(s) => write_text_slot(s, out),
        SlotType::Scalar(s) => write_property(s, out),
        SlotType::Vector(s) => write_property(s, out),
        SlotType::Position(s) => write_property(s, out),
    }
}

fn write_slots_map(slots: &BTreeMap<String, SlotType>, out: &mut String) {
    let mut o = ObjWriter::new(out);
    for (id, slot) in slots {
        let out = o.field(id);
        let mut po = ObjWriter::new(out);
        write_slot(slot, po.field("p"));
        po.finish();
    }
    o.finish();
}

pub(crate) fn slots_to_json_string(slots: &BTreeMap<String, SlotType>) -> String {
    let mut out = String::with_capacity(64 * slots.len() + 2);
    write_slots_map(slots, &mut out);
    out
}

/// Write all slots as batch JSON into the reusable byte buffer (cleared
/// first, allocation reused). A null terminator is NOT appended.
pub(crate) fn slots_to_json_writer(slots: &BTreeMap<String, SlotType>, buf: &mut Vec<u8>) {
    let mut out = String::from_utf8(std::mem::take(buf)).unwrap_or_default();
    out.clear();
    write_slots_map(slots, &mut out);
    *buf = out.into_bytes();
}

pub fn slot_to_json_string(slot: &SlotType) -> String {
    let mut out = String::with_capacity(64);
    write_slot(slot, &mut out);
    out
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
    let v = Value::parse(json).ok()?;
    match slot_type {
        "color" => property_from_json::<ColorValue>(&v).map(SlotType::Color),
        "scalar" => property_from_json::<ScalarValue>(&v).map(SlotType::Scalar),
        "vector" => property_from_json::<[f32; 2]>(&v).map(SlotType::Vector),
        "position" => property_from_json::<[f32; 2]>(&v).map(SlotType::Position),
        "gradient" => gradient_slot_from_json(&v).map(SlotType::Gradient),
        "image" => image_slot_from_json(&v).map(SlotType::Image),
        "text" => text_slot_from_json(&v).map(SlotType::Text),
        _ => None,
    }
}

pub fn slots_from_json_string(json_str: &str) -> Result<BTreeMap<String, SlotType>, super::Error> {
    let root = Value::parse(json_str).map_err(|_| super::Error::InvalidSlotValue)?;
    let pairs = root.as_object().ok_or(super::Error::InvalidSlotValue)?;
    let mut result = BTreeMap::new();
    for (id, value) in pairs {
        if let Some(p) = value.get("p") {
            if let Some(slot_type) = parse_slot_type(p) {
                result.insert(id.to_string(), slot_type);
            }
        }
    }
    Ok(result)
}

pub fn extract_slots_from_animation(animation_json: &str) -> BTreeMap<String, SlotType> {
    let Ok(json) = Value::parse(animation_json) else {
        return BTreeMap::new();
    };

    if let Some(Value::Object(slots_map)) = json.get("slots") {
        let mut result = BTreeMap::new();
        for (id, value) in slots_map {
            if let Some(p) = value.get("p") {
                if let Some(slot_type) = parse_slot_type(p) {
                    result.insert(id.to_string(), slot_type);
                }
            }
        }
        if !result.is_empty() {
            return result;
        }
    }

    let mut result = BTreeMap::new();
    collect_sid_slots(&json, &mut result);
    result
}

/// Walk the JSON tree and collect properties that have a "sid" (slot ID) tag.
/// These are animated properties with an explicit slot identifier, e.g.:
/// `{"a": 0, "k": [0.71, 0.192, 0.278], "sid": "ball_color"}`
fn collect_sid_slots(value: &Value, result: &mut BTreeMap<String, SlotType>) {
    match value {
        Value::Object(pairs) => {
            if let Some(sid) = value.str_field("sid") {
                if let Some(slot_type) = parse_slot_type(value) {
                    result.insert(sid.to_string(), slot_type);
                }
            }
            // Continue walking children regardless
            for (_, v) in pairs {
                collect_sid_slots(v, result);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                collect_sid_slots(v, result);
            }
        }
        _ => {}
    }
}

fn parse_slot_type(value: &Value) -> Option<SlotType> {
    if value.get("w").is_some() || value.get("h").is_some() || value.get("u").is_some() {
        image_slot_from_json(value).map(SlotType::Image)
    } else if value.get("p").is_some() {
        gradient_slot_from_json(value).map(SlotType::Gradient)
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

                        if start_arr.iter().all(Value::is_number) {
                            if len == 3 || len == 4 {
                                return property_from_json::<ColorValue>(value)
                                    .map(SlotType::Color);
                            } else if len == 2 {
                                return property_from_json::<[f32; 2]>(value).map(SlotType::Vector);
                            } else if len == 1 {
                                return property_from_json::<ScalarValue>(value)
                                    .map(SlotType::Scalar);
                            }
                        } else {
                            return text_slot_from_json(value).map(SlotType::Text);
                        }
                    } else if start_value.is_number() {
                        return property_from_json::<ScalarValue>(value).map(SlotType::Scalar);
                    } else if start_value.is_object() {
                        return text_slot_from_json(value).map(SlotType::Text);
                    }
                }
                return None;
            }

            let is_numeric_array = arr.iter().all(Value::is_number);

            if is_numeric_array {
                let len = arr.len();
                if len == 3 || len == 4 {
                    property_from_json::<ColorValue>(value).map(SlotType::Color)
                } else if len == 2 {
                    property_from_json::<[f32; 2]>(value).map(SlotType::Vector)
                } else if len == 1 {
                    property_from_json::<ScalarValue>(value).map(SlotType::Scalar)
                } else {
                    None
                }
            } else {
                // Non-numeric array, treat as text slot
                text_slot_from_json(value).map(SlotType::Text)
            }
        } else if k.is_number() {
            property_from_json::<ScalarValue>(value).map(SlotType::Scalar)
        } else if k.is_object() {
            parse_animated_slot(k)
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_animated_slot(k: &Value) -> Option<SlotType> {
    let k_value = k.get("k")?;
    let arr = k_value.as_array()?;
    if arr.is_empty() {
        return None;
    }

    if (arr.len() == 3 || arr.len() == 4) && arr.iter().all(Value::is_number) {
        property_from_json::<ColorValue>(k).map(SlotType::Color)
    } else if arr.len() == 2 && arr.iter().all(Value::is_number) {
        // 2-element arrays can be either Vector or Position
        // Default to Vector during parsing (theme rules will explicitly create Position when needed)
        property_from_json::<[f32; 2]>(k).map(SlotType::Vector)
    } else if arr.len() == 1 && arr[0].is_number() {
        property_from_json::<ScalarValue>(k).map(SlotType::Scalar)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::Value;
    use crate::renderer::slots::gradient::{gradient_slot_from_json, write_gradient_slot};
    use crate::renderer::slots::image::{image_slot_from_json, write_image_slot};
    use crate::renderer::slots::text::{
        text_document_from_json, text_slot_from_json, write_text_slot,
    };
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
        let anim = r#"{"slots":{"from_slots":{"p":{"a":0,"k":[1.0,0.0,0.0]}}},"layers":[{"ks":{"o":{"a":0,"k":[0.5,0.5,0.5],"sid":"from_sid"}}}]}"#;
        let slots = extract_slots_from_animation(anim);
        assert_eq!(slots.len(), 1);
        assert!(slots.contains_key("from_slots"));
        assert!(!slots.contains_key("from_sid"));
    }

    #[test]
    fn extract_falls_back_to_sid_when_slots_empty() {
        let anim =
            r#"{"slots":{},"layers":[{"ks":{"o":{"a":0,"k":[0.5,0.5,0.5],"sid":"my_color"}}}]}"#;
        let slots = extract_slots_from_animation(anim);
        assert_eq!(slots.len(), 1);
        assert!(slots.contains_key("my_color"));
    }

    #[test]
    fn extract_no_slots_key_uses_sid() {
        let anim = r#"{"layers":[{"ks":{"o":{"a":0,"k":[0.5,0.5,0.5],"sid":"my_color"}}}]}"#;
        let slots = extract_slots_from_animation(anim);
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
        let anim = r#"{"v":"5.0","fr":30,"w":100,"h":100,"layers":[]}"#;
        let slots = extract_slots_from_animation(anim);
        assert!(slots.is_empty());
    }

    // ── Group B: parse_slot_type static branches ──────────────────

    #[test]
    fn parse_static_color_4_elem() {
        let val = Value::parse(r#"{"a":0,"k":[1.0,0.0,0.0,1.0]}"#).unwrap();
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "color");
    }

    #[test]
    fn parse_static_vector_2_elem() {
        let val = Value::parse(r#"{"a":0,"k":[100.0,200.0]}"#).unwrap();
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "vector");
    }

    #[test]
    fn parse_static_scalar_single_elem_array() {
        let val = Value::parse(r#"{"a":0,"k":[42.0]}"#).unwrap();
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "scalar");
    }

    #[test]
    fn parse_image_slot() {
        let val = Value::parse(r#"{"w":100,"h":100,"u":"images/","p":"img.png"}"#).unwrap();
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "image");
    }

    #[test]
    fn parse_gradient_slot() {
        let val =
            Value::parse(r#"{"p":2,"k":{"a":0,"k":[0.0,1.0,0.0,0.0,1.0,0.0,0.0,1.0]}}"#).unwrap();
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "gradient");
    }

    #[test]
    fn parse_empty_k_array_returns_none() {
        let val = Value::parse(r#"{"a":0,"k":[]}"#).unwrap();
        assert!(parse_slot_type(&val).is_none());
    }

    #[test]
    fn parse_no_k_no_w_no_p_returns_none() {
        let val = Value::parse(r#"{"a":0,"x":"something"}"#).unwrap();
        assert!(parse_slot_type(&val).is_none());
    }

    #[test]
    fn parse_k_is_string_returns_none() {
        let val = Value::parse(r#"{"k":"invalid"}"#).unwrap();
        assert!(parse_slot_type(&val).is_none());
    }

    #[test]
    fn parse_5_elem_array_returns_none() {
        let val = Value::parse(r#"{"k":[1.0,2.0,3.0,4.0,5.0]}"#).unwrap();
        assert!(parse_slot_type(&val).is_none());
    }

    // ── Group C: parse_slot_type animated branches ────────────────

    #[test]
    fn parse_animated_color_keyframes() {
        let val =
            Value::parse(r#"{"a":1,"k":[{"t":0,"s":[1.0,0.0,0.0]},{"t":30,"s":[0.0,1.0,0.0]}]}"#)
                .unwrap();
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "color");
    }

    #[test]
    fn parse_animated_scalar_keyframes_number() {
        let val = Value::parse(r#"{"a":1,"k":[{"t":0,"s":50.0},{"t":30,"s":100.0}]}"#).unwrap();
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "scalar");
    }

    #[test]
    fn parse_animated_scalar_keyframes_array() {
        let val = Value::parse(r#"{"a":1,"k":[{"t":0,"s":[50.0]},{"t":30,"s":[100.0]}]}"#).unwrap();
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "scalar");
    }

    #[test]
    fn parse_animated_text_keyframes_object() {
        let val = Value::parse(r#"{"k":[{"t":0,"s":{"t":"Hello"}},{"t":30,"s":{"t":"World"}}]}"#)
            .unwrap();
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "text");
    }

    #[test]
    fn parse_animated_text_keyframes_non_numeric() {
        // When "s" is a non-numeric array, detection tries TextSlot but
        // deserialization fails because TextKeyframe expects "s" to be
        // a TextDocument object, not an array.
        let val = Value::parse(r#"{"k":[{"t":0,"s":["not_a_number"]}]}"#).unwrap();
        assert!(parse_slot_type(&val).is_none());
    }

    // ── Group D: parse_animated_slot (nested k.k pattern) ─────────

    #[test]
    fn parse_nested_animated_color() {
        let val = Value::parse(r#"{"k":{"a":0,"k":[1.0,0.0,0.0]}}"#).unwrap();
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "color");
    }

    #[test]
    fn parse_nested_animated_vector() {
        let val = Value::parse(r#"{"k":{"a":0,"k":[100.0,200.0]}}"#).unwrap();
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "vector");
    }

    #[test]
    fn parse_nested_animated_scalar() {
        let val = Value::parse(r#"{"k":{"a":0,"k":[42.0]}}"#).unwrap();
        let slot = parse_slot_type(&val).unwrap();
        assert_eq!(slot_type_name(&slot), "scalar");
    }

    #[test]
    fn parse_nested_animated_empty_k() {
        let val = Value::parse(r#"{"k":{"a":0,"k":[]}}"#).unwrap();
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
        let json = slots_to_json_string(&map);
        let parsed = slots_from_json_string(&json).unwrap();
        let json2 = slots_to_json_string(&parsed);
        assert_eq!(json, json2);
    }

    #[test]
    fn round_trip_scalar() {
        let mut map = BTreeMap::new();
        map.insert("s".to_string(), SlotType::Scalar(ScalarSlot::new(42.0)));
        let json = slots_to_json_string(&map);
        let parsed = slots_from_json_string(&json).unwrap();
        let json2 = slots_to_json_string(&parsed);
        assert_eq!(json, json2);
    }

    #[test]
    fn round_trip_vector() {
        let mut map = BTreeMap::new();
        map.insert(
            "v".to_string(),
            SlotType::Vector(VectorSlot::static_value([100.0, 200.0])),
        );
        let json = slots_to_json_string(&map);
        let parsed = slots_from_json_string(&json).unwrap();
        let json2 = slots_to_json_string(&parsed);
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
        let json = slots_to_json_string(&map);
        let parsed = slots_from_json_string(&json).unwrap();
        let json2 = slots_to_json_string(&parsed);
        assert_eq!(json, json2);
    }

    #[test]
    fn round_trip_text() {
        let mut map = BTreeMap::new();
        map.insert("txt".to_string(), SlotType::Text(TextSlot::new("Hello")));
        let json = slots_to_json_string(&map);
        let parsed = slots_from_json_string(&json).unwrap();
        let json2 = slots_to_json_string(&parsed);
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
        let json = slots_to_json_string(&map);
        let parsed = slots_from_json_string(&json).unwrap();
        let json2 = slots_to_json_string(&parsed);
        assert_eq!(json, json2);
    }

    #[test]
    fn round_trip_empty_map() {
        let map: BTreeMap<String, SlotType> = BTreeMap::new();
        let json = slots_to_json_string(&map);
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
        let json = slots_to_json_string(&map);
        let parsed = slots_from_json_string(&json).unwrap();
        assert_eq!(parsed.len(), 3);
        let json2 = slots_to_json_string(&parsed);
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
        let json_str = slots_to_json_string(&map);
        let mut buf = Vec::new();
        slots_to_json_writer(&map, &mut buf);
        assert_eq!(json_str.as_bytes(), &buf[..]);
    }

    #[test]
    fn json_writer_clears_buffer() {
        let mut buf = vec![1, 2, 3, 4, 5];
        let map: BTreeMap<String, SlotType> = BTreeMap::new();
        slots_to_json_writer(&map, &mut buf);
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

    #[test]
    fn property_static_color_parses_and_writes() {
        let v = Value::parse(r#"{"a":0,"k":[1,0.25,0],"x":"var(--c)"}"#).unwrap();
        let p = property_from_json::<ColorValue>(&v).unwrap();
        assert!(
            matches!(&p.value, PropertyValue::Static(ColorValue([r, g, b])) if r == &1.0 && g == &0.25 && b == &0.0)
        );
        let mut out = String::new();
        write_property(&p, &mut out);
        assert_eq!(out, r#"{"a":0,"k":[1,0.25,0],"x":"var(--c)"}"#);
    }

    #[test]
    fn property_rgba_input_drops_alpha_on_write() {
        let v = Value::parse(r#"{"a":0,"k":[1,0,0,1]}"#).unwrap();
        let p = property_from_json::<ColorValue>(&v).unwrap();
        let mut out = String::new();
        write_property(&p, &mut out);
        assert_eq!(out, r#"{"a":0,"k":[1,0,0]}"#);
    }

    #[test]
    fn property_animated_keyframes_roundtrip() {
        let src = r#"{"a":1,"k":[{"t":0,"s":[1,0,0],"i":{"x":0.5,"y":[0.1,0.9]},"h":1},{"t":30,"s":[0,1,0],"ti":[1,2],"to":[3,4]}]}"#;
        let v = Value::parse(src).unwrap();
        let p = property_from_json::<ColorValue>(&v).unwrap();
        assert!(matches!(&p.value, PropertyValue::Animated(kfs) if kfs.len() == 2));
        let mut out = String::new();
        write_property(&p, &mut out);
        assert_eq!(out, src);
    }

    #[test]
    fn scalar_value_accepts_number_or_one_array_only() {
        let bare = Value::parse("42.5").unwrap();
        assert_eq!(ScalarValue::from_json(&bare).unwrap().0, 42.5);
        let one = Value::parse("[42.5]").unwrap();
        assert_eq!(ScalarValue::from_json(&one).unwrap().0, 42.5);
        let two = Value::parse("[1,2]").unwrap();
        assert!(ScalarValue::from_json(&two).is_none());
    }

    #[test]
    fn scalar_writes_bare_number() {
        let mut out = String::new();
        write_property(&ScalarSlot::new(42.0), &mut out);
        assert_eq!(out, r#"{"a":0,"k":42}"#);
    }

    #[test]
    fn color_slot_serializes_compact_floats() {
        // serde printed 1.0; the new writer prints integral floats bare.
        let slot = SlotType::Color(ColorSlot::new([1.0, 0.25, 0.0]));
        assert_eq!(slot_to_json_string(&slot), r#"{"a":0,"k":[1,0.25,0]}"#);
    }

    #[test]
    fn gradient_slot_parses_and_writes() {
        let src = r#"{"k":{"a":0,"k":[0,1,0,0,1,0,0,1]},"p":2}"#;
        let v = Value::parse(src).unwrap();
        let g = gradient_slot_from_json(&v).unwrap();
        assert_eq!(g.num_stops, 2);
        let mut out = String::new();
        write_gradient_slot(&g, &mut out);
        assert_eq!(out, src);
    }

    #[test]
    fn image_slot_parses_and_writes_skipping_none() {
        let v = Value::parse(r#"{"w":100,"p":"img.png"}"#).unwrap();
        let img = image_slot_from_json(&v).unwrap();
        assert_eq!(img.width, Some(100));
        assert_eq!(img.height, None);
        let mut out = String::new();
        write_image_slot(&img, &mut out);
        assert_eq!(out, r#"{"w":100,"p":"img.png"}"#);
    }

    #[test]
    fn text_slot_parses_and_writes() {
        let src = r#"{"k":[{"t":0,"s":{"t":"Hi","f":"Arial","s":12,"fc":[1,0,0],"of":true,"j":2,"sz":[100,50]}}]}"#;
        let v = Value::parse(src).unwrap();
        let t = text_slot_from_json(&v).unwrap();
        assert_eq!(t.keyframes[0].text_document.text, "Hi");
        assert_eq!(t.keyframes[0].text_document.justify, Some(2));
        let mut out = String::new();
        write_text_slot(&t, &mut out);
        assert_eq!(out, src);
    }

    #[test]
    fn text_document_requires_t() {
        let v = Value::parse(r#"{"f":"Arial"}"#).unwrap();
        assert!(text_document_from_json(&v).is_none());
    }

    #[test]
    fn property_rejects_wrong_typed_option_field() {
        // "x" present but not a string — serde errors, so must we.
        let v = Value::parse(r#"{"a":0,"k":[1],"x":5}"#).unwrap();
        assert!(property_from_json::<ScalarValue>(&v).is_none());
    }
}
