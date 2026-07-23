use crate::json::{array_of, f32_array, f32_vec, opt, Value};
use crate::renderer::slots::bezier_from_json;
use crate::renderer::slots::{
    Bezier, ColorSlot, ColorValue, GradientSlot, GradientStop, ImageSlot, LottieKeyframe,
    LottieProperty, PositionSlot, ScalarSlot, ScalarValue, SlotType, TextCaps, TextDocument,
    TextJustify, TextKeyframe, TextSlot, VectorSlot,
};
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Theme {
    pub(crate) rules: Vec<ThemeRule>,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("invalid theme")]
    InvalidTheme,
}

impl FromStr for Theme {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let root = Value::parse(s).map_err(|_| Error::InvalidTheme)?;
        let rules = root
            .get("rules")
            .and_then(|r| array_of(r, theme_rule_from_json))
            .ok_or(Error::InvalidTheme)?;
        Ok(Theme { rules })
    }
}

impl Theme {
    pub fn to_slot_types(&self, active_animation_id: &str) -> BTreeMap<String, SlotType> {
        let mut slots = BTreeMap::new();

        for rule in &self.rules {
            if !rule.should_process(active_animation_id) {
                continue;
            }

            if let Some((slot_id, slot_type)) = rule.to_slot() {
                slots.insert(slot_id, slot_type);
            }
        }

        slots
    }

    pub fn get_rule(&self, id: &str) -> Option<&ThemeRule> {
        self.rules.iter().find(|rule| rule.id() == id)
    }

    pub fn get_rule_mut(&mut self, id: &str) -> Option<&mut ThemeRule> {
        self.rules.iter_mut().find(|rule| rule.id() == id)
    }

    pub fn set_rule(&mut self, rule: ThemeRule) {
        let rule_id = rule.id().to_string();
        if let Some(pos) = self
            .rules
            .iter()
            .position(|existing| existing.id() == rule_id)
        {
            self.rules[pos] = rule;
        } else {
            self.rules.push(rule);
        }
    }

    pub fn remove_rule(&mut self, id: &str) -> bool {
        if let Some(pos) = self.rules.iter().position(|rule| rule.id() == id) {
            self.rules.remove(pos);
            true
        } else {
            false
        }
    }
}

// === Typed Theme Rules ===

/// Color theme rule - holds color animation/static values
/// Supports both RGB (3 elements) and RGBA (4 elements) color values
#[derive(Debug, Clone)]
pub struct ColorRule {
    pub id: String,
    pub animations: Option<Vec<String>>,
    pub value: Option<Vec<f32>>,
    pub keyframes: Option<Vec<ColorKeyframe>>,
    pub expression: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColorKeyframe {
    pub frame: u32,
    pub value: Vec<f32>,
    pub in_tangent: Option<Bezier>,
    pub out_tangent: Option<Bezier>,
    pub value_in_tangent: Option<Vec<f32>>,
    pub value_out_tangent: Option<Vec<f32>>,
    pub hold: Option<bool>,
}

/// Scalar theme rule - holds scalar animation/static values
#[derive(Debug, Clone)]
pub struct ScalarRule {
    pub id: String,
    pub animations: Option<Vec<String>>,
    pub value: Option<f32>,
    pub keyframes: Option<Vec<ScalarKeyframe>>,
    pub expression: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScalarKeyframe {
    pub frame: u32,
    pub value: f32,
    pub in_tangent: Option<Bezier>,
    pub out_tangent: Option<Bezier>,
    pub value_in_tangent: Option<Vec<f32>>,
    pub value_out_tangent: Option<Vec<f32>>,
    pub hold: Option<bool>,
}

/// Gradient theme rule - holds gradient animation/static values
#[derive(Debug, Clone)]
pub struct GradientRule {
    pub id: String,
    pub animations: Option<Vec<String>>,
    pub value: Option<Vec<GradientStop>>,
    pub keyframes: Option<Vec<GradientKeyframe>>,
    pub expression: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GradientKeyframe {
    pub frame: u32,
    pub value: Vec<GradientStop>,
    pub in_tangent: Option<Bezier>,
    pub out_tangent: Option<Bezier>,
    pub hold: Option<bool>,
}

/// Image theme rule - holds image asset information
#[derive(Debug, Clone)]
pub struct ImageRule {
    pub id: String,
    pub animations: Option<Vec<String>>,
    pub value: ImageValue,
}

#[derive(Debug, Clone)]
pub struct ImageValue {
    pub src: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Text theme rule - holds text animation/static values
#[derive(Debug, Clone)]
pub struct TextRule {
    pub id: String,
    pub animations: Option<Vec<String>>,
    pub value: Option<TextValue>,
    pub keyframes: Option<Vec<TextRuleKeyframe>>,
    pub expression: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TextValue {
    pub text: String,
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
    pub fill_color: Option<Vec<f32>>,
    pub stroke_color: Option<Vec<f32>>,
    pub stroke_width: Option<f32>,
    pub stroke_over_fill: Option<bool>,
    pub line_height: Option<f32>,
    pub tracking: Option<f32>,
    pub justify: Option<String>,
    pub text_caps: Option<String>,
    pub baseline_shift: Option<f32>,
    pub wrap_size: Option<[f32; 2]>,
    pub wrap_position: Option<[f32; 2]>,
}

#[derive(Debug, Clone)]
pub struct TextRuleKeyframe {
    pub frame: u32,
    pub value: TextValue,
}

/// Vector theme rule - holds 2D vector animation/static values (e.g., scale, size)
#[derive(Debug, Clone)]
pub struct VectorRule {
    pub id: String,
    pub animations: Option<Vec<String>>,
    pub value: Option<Vec<f32>>,
    pub keyframes: Option<Vec<VectorKeyframe>>,
    pub expression: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VectorKeyframe {
    pub frame: u32,
    pub value: Vec<f32>,
    pub in_tangent: Option<Bezier>,
    pub out_tangent: Option<Bezier>,
    pub hold: Option<bool>,
}

/// Position theme rule - holds 2D position animation/static values with spatial tangents
#[derive(Debug, Clone)]
pub struct PositionRule {
    pub id: String,
    pub animations: Option<Vec<String>>,
    pub value: Option<Vec<f32>>,
    pub keyframes: Option<Vec<PositionKeyframe>>,
    pub expression: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PositionKeyframe {
    pub frame: u32,
    pub value: Vec<f32>,
    pub in_tangent: Option<Bezier>,
    pub out_tangent: Option<Bezier>,
    pub value_in_tangent: Option<Vec<f32>>,
    pub value_out_tangent: Option<Vec<f32>>,
    pub hold: Option<bool>,
}

/// Theme rule enum wrapping all rule types
#[derive(Debug, Clone)]
pub enum ThemeRule {
    Color(ColorRule),
    Scalar(ScalarRule),
    Gradient(GradientRule),
    Image(ImageRule),
    Text(TextRule),
    Vector(VectorRule),
    Position(PositionRule),
}

impl ThemeRule {
    pub fn id(&self) -> &str {
        match self {
            ThemeRule::Color(r) => &r.id,
            ThemeRule::Scalar(r) => &r.id,
            ThemeRule::Gradient(r) => &r.id,
            ThemeRule::Image(r) => &r.id,
            ThemeRule::Text(r) => &r.id,
            ThemeRule::Vector(r) => &r.id,
            ThemeRule::Position(r) => &r.id,
        }
    }

    pub fn animations(&self) -> Option<&Vec<String>> {
        match self {
            ThemeRule::Color(r) => r.animations.as_ref(),
            ThemeRule::Scalar(r) => r.animations.as_ref(),
            ThemeRule::Gradient(r) => r.animations.as_ref(),
            ThemeRule::Image(r) => r.animations.as_ref(),
            ThemeRule::Text(r) => r.animations.as_ref(),
            ThemeRule::Vector(r) => r.animations.as_ref(),
            ThemeRule::Position(r) => r.animations.as_ref(),
        }
    }

    pub fn should_process(&self, active_animation_id: &str) -> bool {
        match self.animations() {
            None => true,
            Some(animations) => animations.iter().any(|anim| anim == active_animation_id),
        }
    }

    pub fn to_slot(&self) -> Option<(String, SlotType)> {
        let slot_id = self.id().to_string();
        let slot = match self {
            ThemeRule::Color(rule) => SlotType::Color(ColorSlot::from(rule)),
            ThemeRule::Scalar(rule) => SlotType::Scalar(ScalarSlot::from(rule)),
            ThemeRule::Gradient(rule) => SlotType::Gradient(GradientSlot::from(rule)),
            ThemeRule::Image(rule) => SlotType::Image(ImageSlot::from(rule)),
            ThemeRule::Text(rule) => SlotType::Text(TextSlot::from(rule)),
            ThemeRule::Vector(rule) => SlotType::Vector(VectorSlot::from(rule)),
            ThemeRule::Position(rule) => SlotType::Position(PositionSlot::from(rule)),
        };
        Some((slot_id, slot))
    }
}

fn theme_rule_from_json(v: &Value) -> Option<ThemeRule> {
    Some(match v.str_field("type")? {
        "Color" => ThemeRule::Color(color_rule_from_json(v)?),
        "Scalar" => ThemeRule::Scalar(scalar_rule_from_json(v)?),
        "Gradient" => ThemeRule::Gradient(gradient_rule_from_json(v)?),
        "Image" => ThemeRule::Image(image_rule_from_json(v)?),
        "Text" => ThemeRule::Text(text_rule_from_json(v)?),
        "Vector" => ThemeRule::Vector(vector_rule_from_json(v)?),
        "Position" => ThemeRule::Position(position_rule_from_json(v)?),
        _ => return None,
    })
}

fn string_list(v: &Value) -> Option<Vec<String>> {
    array_of(v, |s| s.as_str().map(str::to_owned))
}

fn opt_animations(v: &Value) -> Option<Option<Vec<String>>> {
    opt(v.get("animations"), string_list)
}

fn opt_expression(v: &Value) -> Option<Option<String>> {
    v.opt_str_field("expression")
}

fn opt_keyframes<T>(v: &Value, parse: impl Fn(&Value) -> Option<T>) -> Option<Option<Vec<T>>> {
    opt(v.get("keyframes"), |k| array_of(k, &parse))
}

/// The field block shared by every theme keyframe shape.
struct KeyframeFields<T> {
    frame: u32,
    value: T,
    in_tangent: Option<Bezier>,
    out_tangent: Option<Bezier>,
    hold: Option<bool>,
}

fn keyframe_fields<'a, T>(
    v: &Value<'a>,
    parse_value: impl Fn(&Value<'a>) -> Option<T>,
) -> Option<KeyframeFields<T>> {
    Some(KeyframeFields {
        frame: v.u32_field("frame")?,
        value: parse_value(v.get("value")?)?,
        in_tangent: opt(v.get("inTangent"), bezier_from_json)?,
        out_tangent: opt(v.get("outTangent"), bezier_from_json)?,
        hold: opt(v.get("hold"), Value::as_bool)?,
    })
}

type ValueTangents = (Option<Vec<f32>>, Option<Vec<f32>>);

fn value_tangents(v: &Value) -> Option<ValueTangents> {
    Some((
        opt(v.get("valueInTangent"), f32_vec)?,
        opt(v.get("valueOutTangent"), f32_vec)?,
    ))
}

fn rule_id(v: &Value) -> Option<String> {
    v.str_field("id").map(str::to_owned)
}

fn color_rule_from_json(v: &Value) -> Option<ColorRule> {
    Some(ColorRule {
        id: rule_id(v)?,
        animations: opt_animations(v)?,
        value: opt(v.get("value"), f32_vec)?,
        keyframes: opt_keyframes(v, color_keyframe_from_json)?,
        expression: opt_expression(v)?,
    })
}

fn color_keyframe_from_json(v: &Value) -> Option<ColorKeyframe> {
    let kf = keyframe_fields(v, f32_vec)?;
    let (value_in_tangent, value_out_tangent) = value_tangents(v)?;
    Some(ColorKeyframe {
        frame: kf.frame,
        value: kf.value,
        in_tangent: kf.in_tangent,
        out_tangent: kf.out_tangent,
        value_in_tangent,
        value_out_tangent,
        hold: kf.hold,
    })
}

fn scalar_rule_from_json(v: &Value) -> Option<ScalarRule> {
    Some(ScalarRule {
        id: rule_id(v)?,
        animations: opt_animations(v)?,
        value: opt(v.get("value"), Value::as_f32)?,
        keyframes: opt_keyframes(v, scalar_keyframe_from_json)?,
        expression: opt_expression(v)?,
    })
}

fn scalar_keyframe_from_json(v: &Value) -> Option<ScalarKeyframe> {
    let kf = keyframe_fields(v, Value::as_f32)?;
    let (value_in_tangent, value_out_tangent) = value_tangents(v)?;
    Some(ScalarKeyframe {
        frame: kf.frame,
        value: kf.value,
        in_tangent: kf.in_tangent,
        out_tangent: kf.out_tangent,
        value_in_tangent,
        value_out_tangent,
        hold: kf.hold,
    })
}

fn gradient_stop_from_json(v: &Value) -> Option<GradientStop> {
    Some(GradientStop {
        offset: v.f32_field("offset")?,
        color: f32_array::<4>(v.get("color")?)?,
    })
}

fn gradient_stops(v: &Value) -> Option<Vec<GradientStop>> {
    array_of(v, gradient_stop_from_json)
}

fn gradient_rule_from_json(v: &Value) -> Option<GradientRule> {
    Some(GradientRule {
        id: rule_id(v)?,
        animations: opt_animations(v)?,
        value: opt(v.get("value"), gradient_stops)?,
        keyframes: opt_keyframes(v, |kf| {
            let kf = keyframe_fields(kf, gradient_stops)?;
            Some(GradientKeyframe {
                frame: kf.frame,
                value: kf.value,
                in_tangent: kf.in_tangent,
                out_tangent: kf.out_tangent,
                hold: kf.hold,
            })
        })?,
        expression: opt_expression(v)?,
    })
}

fn image_rule_from_json(v: &Value) -> Option<ImageRule> {
    let value = v.get("value")?;
    Some(ImageRule {
        id: rule_id(v)?,
        animations: opt_animations(v)?,
        value: ImageValue {
            src: value.str_field("src")?.to_owned(),
            width: opt(value.get("width"), Value::as_u32)?,
            height: opt(value.get("height"), Value::as_u32)?,
        },
    })
}

fn text_value_from_json(v: &Value) -> Option<TextValue> {
    Some(TextValue {
        text: v.str_field("text")?.to_owned(),
        font_name: v.opt_str_field("fontName")?,
        font_size: opt(v.get("fontSize"), Value::as_f32)?,
        fill_color: opt(v.get("fillColor"), f32_vec)?,
        stroke_color: opt(v.get("strokeColor"), f32_vec)?,
        stroke_width: opt(v.get("strokeWidth"), Value::as_f32)?,
        stroke_over_fill: opt(v.get("strokeOverFill"), Value::as_bool)?,
        line_height: opt(v.get("lineHeight"), Value::as_f32)?,
        tracking: opt(v.get("tracking"), Value::as_f32)?,
        justify: v.opt_str_field("justify")?,
        text_caps: v.opt_str_field("textCaps")?,
        baseline_shift: opt(v.get("baselineShift"), Value::as_f32)?,
        wrap_size: opt(v.get("wrapSize"), f32_array::<2>)?,
        wrap_position: opt(v.get("wrapPosition"), f32_array::<2>)?,
    })
}

fn text_rule_from_json(v: &Value) -> Option<TextRule> {
    Some(TextRule {
        id: rule_id(v)?,
        animations: opt_animations(v)?,
        value: opt(v.get("value"), text_value_from_json)?,
        keyframes: opt_keyframes(v, |kf| {
            Some(TextRuleKeyframe {
                frame: kf.u32_field("frame")?,
                value: text_value_from_json(kf.get("value")?)?,
            })
        })?,
        expression: opt_expression(v)?,
    })
}

fn vector_rule_from_json(v: &Value) -> Option<VectorRule> {
    Some(VectorRule {
        id: rule_id(v)?,
        animations: opt_animations(v)?,
        value: opt(v.get("value"), f32_vec)?,
        keyframes: opt_keyframes(v, |kf| {
            let kf = keyframe_fields(kf, f32_vec)?;
            Some(VectorKeyframe {
                frame: kf.frame,
                value: kf.value,
                in_tangent: kf.in_tangent,
                out_tangent: kf.out_tangent,
                hold: kf.hold,
            })
        })?,
        expression: opt_expression(v)?,
    })
}

fn position_rule_from_json(v: &Value) -> Option<PositionRule> {
    Some(PositionRule {
        id: rule_id(v)?,
        animations: opt_animations(v)?,
        value: opt(v.get("value"), f32_vec)?,
        keyframes: opt_keyframes(v, |kf| {
            let (value_in_tangent, value_out_tangent) = value_tangents(kf)?;
            let kf = keyframe_fields(kf, f32_vec)?;
            Some(PositionKeyframe {
                frame: kf.frame,
                value: kf.value,
                in_tangent: kf.in_tangent,
                out_tangent: kf.out_tangent,
                value_in_tangent,
                value_out_tangent,
                hold: kf.hold,
            })
        })?,
        expression: opt_expression(v)?,
    })
}

pub fn transform_theme_to_lottie_slots(theme_json: &str, active_animation_id: &str) -> String {
    match theme_json.parse::<Theme>() {
        Ok(theme) => {
            let slots = theme.to_slot_types(active_animation_id);
            crate::renderer::slots::slots_to_json_string(&slots)
        }
        Err(_) => String::new(),
    }
}

// === Rule -> Slot conversions ===

impl From<&ColorRule> for ColorSlot {
    fn from(rule: &ColorRule) -> Self {
        if let Some(keyframes) = &rule.keyframes {
            let lottie_keyframes: Vec<LottieKeyframe<ColorValue>> = keyframes
                .iter()
                .map(|kf| {
                    // Support both RGB (3) and RGBA (4) formats, extract RGB
                    let rgb = if kf.value.len() >= 3 {
                        ColorValue([kf.value[0], kf.value[1], kf.value[2]])
                    } else {
                        ColorValue([0.0, 0.0, 0.0])
                    };

                    LottieKeyframe {
                        frame: kf.frame,
                        start_value: rgb,
                        in_tangent: kf.in_tangent.clone(),
                        out_tangent: kf.out_tangent.clone(),
                        value_in_tangent: kf.value_in_tangent.clone(),
                        value_out_tangent: kf.value_out_tangent.clone(),
                        hold: kf.hold.map(|b| if b { 1 } else { 0 }),
                    }
                })
                .collect();

            let mut slot = LottieProperty::animated(lottie_keyframes);
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else if let Some(value) = &rule.value {
            // Support both RGB (3) and RGBA (4) formats, extract RGB
            let rgb_value = if value.len() >= 3 {
                ColorValue([value[0], value[1], value[2]])
            } else {
                ColorValue([0.0, 0.0, 0.0])
            };

            let mut slot = LottieProperty::static_value(rgb_value);
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else {
            LottieProperty::static_value(ColorValue([0.0, 0.0, 0.0]))
        }
    }
}

impl From<&ScalarRule> for ScalarSlot {
    fn from(rule: &ScalarRule) -> Self {
        if let Some(keyframes) = &rule.keyframes {
            let lottie_keyframes: Vec<LottieKeyframe<ScalarValue>> = keyframes
                .iter()
                .map(|kf| LottieKeyframe {
                    frame: kf.frame,
                    start_value: ScalarValue(kf.value),
                    in_tangent: kf.in_tangent.clone(),
                    out_tangent: kf.out_tangent.clone(),
                    value_in_tangent: kf.value_in_tangent.clone(),
                    value_out_tangent: kf.value_out_tangent.clone(),
                    hold: kf.hold.map(|b| if b { 1 } else { 0 }),
                })
                .collect();

            let mut slot = LottieProperty::animated(lottie_keyframes);
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else if let Some(value) = rule.value {
            let mut slot = LottieProperty::static_value(ScalarValue(value));
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else {
            LottieProperty::static_value(ScalarValue(0.0))
        }
    }
}

impl From<&GradientRule> for GradientSlot {
    fn from(rule: &GradientRule) -> Self {
        if let Some(keyframes) = &rule.keyframes {
            let lottie_keyframes: Vec<LottieKeyframe<Vec<GradientStop>>> = keyframes
                .iter()
                .map(|kf| LottieKeyframe {
                    frame: kf.frame,
                    start_value: kf.value.clone(),
                    in_tangent: kf.in_tangent.clone(),
                    out_tangent: kf.out_tangent.clone(),
                    value_in_tangent: None,
                    value_out_tangent: None,
                    hold: kf.hold.map(|b| if b { 1 } else { 0 }),
                })
                .collect();

            let mut slot = GradientSlot::with_keyframes(lottie_keyframes);
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else if let Some(value) = &rule.value {
            let mut slot = GradientSlot::new(value.clone());
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else {
            GradientSlot::new(vec![])
        }
    }
}

impl From<&ImageRule> for ImageSlot {
    fn from(rule: &ImageRule) -> Self {
        let value = &rule.value;
        let mut slot = ImageSlot::from_src(value.src.clone());

        if let (Some(w), Some(h)) = (value.width, value.height) {
            slot = slot.with_dimensions(w, h);
        }

        slot
    }
}

impl From<&TextRule> for TextSlot {
    fn from(rule: &TextRule) -> Self {
        if let Some(keyframes) = &rule.keyframes {
            let text_keyframes: Vec<TextKeyframe> = keyframes
                .iter()
                .map(|kf| TextKeyframe {
                    frame: kf.frame,
                    text_document: text_value_to_document(&kf.value),
                })
                .collect();

            let mut slot = TextSlot::with_keyframes(text_keyframes);
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else if let Some(value) = &rule.value {
            let document = text_value_to_document(value);
            let mut slot = TextSlot::with_document(document);
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else {
            TextSlot::new("")
        }
    }
}

impl From<&VectorRule> for VectorSlot {
    fn from(rule: &VectorRule) -> Self {
        if let Some(keyframes) = &rule.keyframes {
            let lottie_keyframes: Vec<LottieKeyframe<[f32; 2]>> = keyframes
                .iter()
                .map(|kf| {
                    // Extract 2D vector [x, y]
                    let vec2 = if kf.value.len() >= 2 {
                        [kf.value[0], kf.value[1]]
                    } else {
                        [0.0, 0.0]
                    };

                    LottieKeyframe {
                        frame: kf.frame,
                        start_value: vec2,
                        in_tangent: kf.in_tangent.clone(),
                        out_tangent: kf.out_tangent.clone(),
                        value_in_tangent: None, // Vector typically doesn't use spatial tangents
                        value_out_tangent: None,
                        hold: kf.hold.map(|b| if b { 1 } else { 0 }),
                    }
                })
                .collect();

            let mut slot = LottieProperty::animated(lottie_keyframes);
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else if let Some(value) = &rule.value {
            // Extract 2D vector [x, y]
            let vec2_value = if value.len() >= 2 {
                [value[0], value[1]]
            } else {
                [0.0, 0.0]
            };

            let mut slot = LottieProperty::static_value(vec2_value);
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else {
            LottieProperty::static_value([0.0, 0.0])
        }
    }
}

impl From<&PositionRule> for PositionSlot {
    fn from(rule: &PositionRule) -> Self {
        if let Some(keyframes) = &rule.keyframes {
            let lottie_keyframes: Vec<LottieKeyframe<[f32; 2]>> = keyframes
                .iter()
                .map(|kf| {
                    // Extract 2D position [x, y]
                    let pos2 = if kf.value.len() >= 2 {
                        [kf.value[0], kf.value[1]]
                    } else {
                        [0.0, 0.0]
                    };

                    LottieKeyframe {
                        frame: kf.frame,
                        start_value: pos2,
                        in_tangent: kf.in_tangent.clone(),
                        out_tangent: kf.out_tangent.clone(),
                        value_in_tangent: kf.value_in_tangent.clone(), // Position uses spatial tangents
                        value_out_tangent: kf.value_out_tangent.clone(),
                        hold: kf.hold.map(|b| if b { 1 } else { 0 }),
                    }
                })
                .collect();

            let mut slot = LottieProperty::animated(lottie_keyframes);
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else if let Some(value) = &rule.value {
            // Extract 2D position [x, y]
            let pos2_value = if value.len() >= 2 {
                [value[0], value[1]]
            } else {
                [0.0, 0.0]
            };

            let mut slot = LottieProperty::static_value(pos2_value);
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else {
            LottieProperty::static_value([0.0, 0.0])
        }
    }
}

fn text_value_to_document(value: &TextValue) -> TextDocument {
    TextDocument {
        text: value.text.clone(),
        font_name: value.font_name.clone(),
        font_size: value.font_size,
        fill_color: value.fill_color.clone(),
        stroke_color: value.stroke_color.clone(),
        stroke_width: value.stroke_width,
        stroke_over_fill: value.stroke_over_fill,
        line_height: value.line_height,
        tracking: value.tracking,
        justify: value
            .justify
            .as_ref()
            .and_then(|j| parse_justify(j))
            .map(|j| j.to_number()),
        text_caps: value
            .text_caps
            .as_ref()
            .and_then(|c| parse_caps(c))
            .map(|c| c.to_number()),
        baseline_shift: value.baseline_shift,
        wrap_size: value.wrap_size,
        wrap_position: value.wrap_position,
    }
}

#[inline]
fn parse_justify(justify: &str) -> Option<TextJustify> {
    match justify {
        "Left" => Some(TextJustify::Left),
        "Right" => Some(TextJustify::Right),
        "Center" => Some(TextJustify::Center),
        "JustifyLastLeft" => Some(TextJustify::JustifyLastLeft),
        "JustifyLastRight" => Some(TextJustify::JustifyLastRight),
        "JustifyLastCenter" => Some(TextJustify::JustifyLastCenter),
        "JustifyLastFull" => Some(TextJustify::JustifyLastFull),
        _ => None,
    }
}

#[inline]
fn parse_caps(caps: &str) -> Option<TextCaps> {
    match caps {
        "Regular" => Some(TextCaps::Regular),
        "AllCaps" => Some(TextCaps::AllCaps),
        "SmallCaps" => Some(TextCaps::SmallCaps),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::Value;

    fn slot_from_rule(json: &str) -> ImageSlot {
        let v = Value::parse(json).unwrap();
        let ThemeRule::Image(rule) = theme_rule_from_json(&v).expect("valid image rule") else {
            panic!("expected image rule");
        };
        ImageSlot::from(&rule)
    }

    #[test]
    fn image_src_data_url_is_embedded() {
        let slot = slot_from_rule(
            r#"{ "type": "Image", "id": "logo", "value": { "src": "data:image/png;base64,AAAA" } }"#,
        );
        assert_eq!(slot.embed, Some(1));
        assert_eq!(slot.path.as_deref(), Some("data:image/png;base64,AAAA"));
        assert_eq!(slot.directory, None);
    }

    #[test]
    fn image_src_http_url_is_linked_and_intact() {
        let slot = slot_from_rule(
            r#"{ "type": "Image", "id": "logo", "value": { "src": "https://cdn.x/a/logo.png" } }"#,
        );
        assert_eq!(slot.embed, Some(0));
        assert_eq!(slot.path.as_deref(), Some("https://cdn.x/a/logo.png"));
        assert_eq!(slot.directory, None);
    }

    #[test]
    fn image_src_bare_name_resolves_to_package_file() {
        let slot = slot_from_rule(
            r#"{ "type": "Image", "id": "logo", "value": { "src": "logo_dark.png" } }"#,
        );
        assert_eq!(slot.embed, Some(0));
        assert_eq!(slot.path.as_deref(), Some("logo_dark.png"));
        assert_eq!(slot.directory, None);
    }

    #[test]
    fn image_dimensions_are_applied() {
        let slot = slot_from_rule(
            r#"{ "type": "Image", "id": "logo", "value": { "src": "logo.png", "width": 200, "height": 100 } }"#,
        );
        assert_eq!(slot.width, Some(200));
        assert_eq!(slot.height, Some(100));
    }

    #[test]
    fn image_rule_requires_src() {
        let v = Value::parse(r#"{ "type": "Image", "id": "logo", "value": {} }"#).unwrap();
        assert!(
            theme_rule_from_json(&v).is_none(),
            "no src must be rejected"
        );
    }

    #[test]
    fn full_theme_parses_all_rule_types() {
        let theme: Theme = r#"{"rules":[
            {"type":"Color","id":"c","value":[1,0,0],"animations":["a1"]},
            {"type":"Color","id":"ck","keyframes":[{"frame":0,"value":[1,0,0,1],"inTangent":{"x":0.5,"y":0.5},"hold":true}]},
            {"type":"Scalar","id":"s","value":40.5,"expression":"x"},
            {"type":"Gradient","id":"g","value":[{"offset":0,"color":[1,0,0,1]},{"offset":1,"color":[0,0,1,0.5]}]},
            {"type":"Image","id":"i","value":{"src":"x.png","width":10,"height":20}},
            {"type":"Text","id":"t","value":{"text":"hi","fontSize":12,"justify":"Center","wrapSize":[100,50]}},
            {"type":"Vector","id":"v","value":[3,4]},
            {"type":"Position","id":"p","keyframes":[{"frame":0,"value":[1,2],"valueInTangent":[0.1,0.2]}]}
        ]}"#
            .parse()
            .expect("theme parses");
        assert_eq!(theme.rules.len(), 8);
        assert!(theme.get_rule("c").is_some());
        // Rule targeting: "c" only applies to a1.
        assert_eq!(theme.to_slot_types("a1").len(), 8);
        assert_eq!(theme.to_slot_types("other").len(), 7);
    }

    #[test]
    fn unknown_rule_type_rejects_whole_theme() {
        assert!(r#"{"rules":[{"type":"Nope","id":"x"}]}"#.parse::<Theme>().is_err());
    }

    #[test]
    fn missing_rules_key_rejects() {
        assert!("{}".parse::<Theme>().is_err());
        assert!("not json".parse::<Theme>().is_err());
    }

    #[test]
    fn gradient_stop_requires_four_color_components() {
        for bad in [
            r#"[{"offset":0,"color":[1,0,0]}]"#,
            r#"[{"offset":0,"color":[1,0,0,1,0]}]"#,
        ] {
            let json = format!(r#"{{"rules":[{{"type":"Gradient","id":"g","value":{bad}}}]}}"#);
            assert!(
                json.parse::<Theme>().is_err(),
                "should reject color len != 4"
            );
        }
    }
}
