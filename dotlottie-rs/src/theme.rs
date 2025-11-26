use crate::lottie_renderer::slots::{
    Bezier, ColorSlot, GradientSlot, GradientStop, ImageSlot, LottieKeyframe, LottieProperty,
    PositionSlot, ScalarSlot, SlotType, TextCaps, TextDocument, TextJustify, TextKeyframe,
    TextSlot, VectorSlot,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub(crate) rules: Vec<ThemeRule>,
}

impl FromStr for Theme {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
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
        if let Some(pos) = self.rules.iter().position(|existing| existing.id() == rule_id) {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorRule {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyframes: Option<Vec<ColorKeyframe>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorKeyframe {
    pub frame: u32,
    pub value: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_in_tangent: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_out_tangent: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<bool>,
}

/// Scalar theme rule - holds scalar animation/static values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalarRule {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyframes: Option<Vec<ScalarKeyframe>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalarKeyframe {
    pub frame: u32,
    pub value: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_in_tangent: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_out_tangent: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<bool>,
}

/// Gradient theme rule - holds gradient animation/static values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradientRule {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Vec<GradientStop>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyframes: Option<Vec<GradientKeyframe>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradientKeyframe {
    pub frame: u32,
    pub value: Vec<GradientStop>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<bool>,
}

/// Image theme rule - holds image asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageRule {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<Vec<String>>,
    pub value: ImageValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageValue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_url: Option<String>,
}

/// Text theme rule - holds text animation/static values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRule {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<TextValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyframes: Option<Vec<TextRuleKeyframe>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextValue {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_over_fill: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_caps: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_shift: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_size: Option<[f32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_position: Option<[f32; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRuleKeyframe {
    pub frame: u32,
    pub value: TextValue,
}

/// Vector theme rule - holds 2D vector animation/static values (e.g., scale, size)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VectorRule {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyframes: Option<Vec<VectorKeyframe>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VectorKeyframe {
    pub frame: u32,
    pub value: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<bool>,
}

/// Position theme rule - holds 2D position animation/static values with spatial tangents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionRule {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyframes: Option<Vec<PositionKeyframe>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionKeyframe {
    pub frame: u32,
    pub value: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_tangent: Option<Bezier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_in_tangent: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_out_tangent: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<bool>,
}

/// Theme rule enum wrapping all rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
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

pub fn transform_theme_to_lottie_slots(
    theme_json: &str,
    active_animation_id: &str,
) -> String {
    match theme_json.parse::<Theme>() {
        Ok(theme) => {
            let slots = theme.to_slot_types(active_animation_id);
            crate::lottie_renderer::slots::slots_to_json_string(&slots).unwrap_or_default()
        }
        Err(_) => String::new(),
    }
}

// === Rule -> Slot conversions ===

impl From<&ColorRule> for ColorSlot {
    fn from(rule: &ColorRule) -> Self {
        if let Some(keyframes) = &rule.keyframes {
            let lottie_keyframes: Vec<LottieKeyframe<[f32; 3]>> = keyframes
                .iter()
                .map(|kf| {
                    // Support both RGB (3) and RGBA (4) formats, extract RGB
                    let rgb = if kf.value.len() >= 3 {
                        [kf.value[0], kf.value[1], kf.value[2]]
                    } else {
                        [0.0, 0.0, 0.0]
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
                [value[0], value[1], value[2]]
            } else {
                [0.0, 0.0, 0.0]
            };

            let mut slot = LottieProperty::static_value(rgb_value);
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else {
            LottieProperty::static_value([0.0, 0.0, 0.0])
        }
    }
}

impl From<&ScalarRule> for ScalarSlot {
    fn from(rule: &ScalarRule) -> Self {
        if let Some(keyframes) = &rule.keyframes {
            let lottie_keyframes: Vec<LottieKeyframe<f32>> = keyframes
                .iter()
                .map(|kf| LottieKeyframe {
                    frame: kf.frame,
                    start_value: kf.value,
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
            let mut slot = LottieProperty::static_value(value);
            if let Some(expr) = &rule.expression {
                slot = slot.with_expression(expr.clone());
            }
            slot
        } else {
            LottieProperty::static_value(0.0)
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
        let mut slot = if let Some(data_url) = &value.data_url {
            ImageSlot::from_data_url(data_url.clone())
        } else if let Some(path) = &value.path {
            ImageSlot::from_path(path.clone())
        } else {
            ImageSlot {
                width: None,
                height: None,
                directory: None,
                path: None,
                embed: None,
            }
        };

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
        justify: value.justify.as_ref().and_then(|j| parse_justify(j)).map(|j| j.to_number()),
        text_caps: value.text_caps.as_ref().and_then(|c| parse_caps(c)).map(|c| c.to_number()),
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
