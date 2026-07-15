use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextJustify {
    Left,
    Right,
    Center,
    JustifyLastLeft,
    JustifyLastRight,
    JustifyLastCenter,
    JustifyLastFull,
}

impl TextJustify {
    #[inline]
    pub fn to_number(&self) -> u8 {
        match self {
            TextJustify::Left => 0,
            TextJustify::Right => 1,
            TextJustify::Center => 2,
            TextJustify::JustifyLastLeft => 3,
            TextJustify::JustifyLastRight => 4,
            TextJustify::JustifyLastCenter => 5,
            TextJustify::JustifyLastFull => 6,
        }
    }

    #[inline]
    pub fn from_number(value: u8) -> Option<Self> {
        match value {
            0 => Some(TextJustify::Left),
            1 => Some(TextJustify::Right),
            2 => Some(TextJustify::Center),
            3 => Some(TextJustify::JustifyLastLeft),
            4 => Some(TextJustify::JustifyLastRight),
            5 => Some(TextJustify::JustifyLastCenter),
            6 => Some(TextJustify::JustifyLastFull),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextCaps {
    Regular,
    AllCaps,
    SmallCaps,
}

impl TextCaps {
    #[inline]
    pub fn to_number(&self) -> u8 {
        match self {
            TextCaps::Regular => 0,
            TextCaps::AllCaps => 1,
            TextCaps::SmallCaps => 2,
        }
    }

    #[inline]
    pub fn from_number(value: u8) -> Option<Self> {
        match value {
            0 => Some(TextCaps::Regular),
            1 => Some(TextCaps::AllCaps),
            2 => Some(TextCaps::SmallCaps),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocument {
    #[serde(rename = "t")]
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "f")]
    pub font_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "s")]
    pub font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "fc")]
    pub fill_color: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "sc")]
    pub stroke_color: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "sw")]
    pub stroke_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "of")]
    pub stroke_over_fill: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "lh")]
    pub line_height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "tr")]
    pub tracking: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "j")]
    pub justify: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "ca")]
    pub text_caps: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "ls")]
    pub baseline_shift: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "sz")]
    pub wrap_size: Option<[f32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "ps")]
    pub wrap_position: Option<[f32; 2]>,
}

impl TextDocument {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_name: None,
            font_size: None,
            fill_color: None,
            stroke_color: None,
            stroke_width: None,
            stroke_over_fill: None,
            line_height: None,
            tracking: None,
            justify: None,
            text_caps: None,
            baseline_shift: None,
            wrap_size: None,
            wrap_position: None,
        }
    }

    pub fn with_font(mut self, font: impl Into<String>) -> Self {
        self.font_name = Some(font.into());
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    pub fn with_fill_color(mut self, color: Vec<f32>) -> Self {
        self.fill_color = Some(color);
        self
    }

    pub fn with_stroke_color(mut self, color: Vec<f32>) -> Self {
        self.stroke_color = Some(color);
        self
    }

    pub fn with_stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = Some(width);
        self
    }

    pub fn with_stroke_over_fill(mut self, stroke_over_fill: bool) -> Self {
        self.stroke_over_fill = Some(stroke_over_fill);
        self
    }

    pub fn with_line_height(mut self, height: f32) -> Self {
        self.line_height = Some(height);
        self
    }

    pub fn with_tracking(mut self, tracking: f32) -> Self {
        self.tracking = Some(tracking);
        self
    }

    pub fn with_justify(mut self, justify: TextJustify) -> Self {
        self.justify = Some(justify.to_number());
        self
    }

    pub fn with_caps(mut self, caps: TextCaps) -> Self {
        self.text_caps = Some(caps.to_number());
        self
    }

    pub fn with_baseline_shift(mut self, shift: f32) -> Self {
        self.baseline_shift = Some(shift);
        self
    }

    pub fn with_wrap_size(mut self, size: [f32; 2]) -> Self {
        self.wrap_size = Some(size);
        self
    }

    pub fn with_wrap_position(mut self, position: [f32; 2]) -> Self {
        self.wrap_position = Some(position);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextKeyframe {
    #[serde(rename = "t")]
    pub frame: u32,
    #[serde(rename = "s")]
    pub text_document: TextDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSlot {
    #[serde(rename = "k")]
    pub keyframes: Vec<TextKeyframe>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "x")]
    pub expression: Option<String>,
}

impl TextSlot {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            keyframes: vec![TextKeyframe {
                frame: 0,
                text_document: TextDocument::new(text),
            }],
            expression: None,
        }
    }

    pub fn with_document(document: TextDocument) -> Self {
        Self {
            keyframes: vec![TextKeyframe {
                frame: 0,
                text_document: document,
            }],
            expression: None,
        }
    }

    pub fn with_keyframes(keyframes: Vec<TextKeyframe>) -> Self {
        Self {
            keyframes,
            expression: None,
        }
    }

    pub fn with_expression(mut self, expr: String) -> Self {
        self.expression = Some(expr);
        self
    }
}
