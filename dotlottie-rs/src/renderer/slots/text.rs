use crate::json::{
    array_of, f32_array, f32_vec, opt, write_f32, write_seq, write_str, ObjWriter, Value,
};
use crate::renderer::slots::write_f32_slice;
use std::fmt::Write as _;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct TextDocument {
    pub text: String,
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
    pub fill_color: Option<Vec<f32>>,
    pub stroke_color: Option<Vec<f32>>,
    pub stroke_width: Option<f32>,
    pub stroke_over_fill: Option<bool>,
    pub line_height: Option<f32>,
    pub tracking: Option<f32>,
    pub justify: Option<u8>,
    pub text_caps: Option<u8>,
    pub baseline_shift: Option<f32>,
    pub wrap_size: Option<[f32; 2]>,
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

#[derive(Debug, Clone)]
pub struct TextKeyframe {
    pub frame: u32,
    pub text_document: TextDocument,
}

#[derive(Debug, Clone)]
pub struct TextSlot {
    pub keyframes: Vec<TextKeyframe>,
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

pub(crate) fn text_document_from_json(v: &Value) -> Option<TextDocument> {
    Some(TextDocument {
        text: v.str_field("t")?.to_owned(),
        font_name: v.opt_str_field("f")?,
        font_size: opt(v.get("s"), Value::as_f32)?,
        fill_color: opt(v.get("fc"), f32_vec)?,
        stroke_color: opt(v.get("sc"), f32_vec)?,
        stroke_width: opt(v.get("sw"), Value::as_f32)?,
        stroke_over_fill: opt(v.get("of"), Value::as_bool)?,
        line_height: opt(v.get("lh"), Value::as_f32)?,
        tracking: opt(v.get("tr"), Value::as_f32)?,
        justify: opt(v.get("j"), Value::as_u8)?,
        text_caps: opt(v.get("ca"), Value::as_u8)?,
        baseline_shift: opt(v.get("ls"), Value::as_f32)?,
        wrap_size: opt(v.get("sz"), f32_array::<2>)?,
        wrap_position: opt(v.get("ps"), f32_array::<2>)?,
    })
}

fn write_text_document(d: &TextDocument, out: &mut String) {
    let mut o = ObjWriter::new(out);
    write_str(&d.text, o.field("t"));
    if let Some(f) = &d.font_name {
        write_str(f, o.field("f"));
    }
    if let Some(s) = d.font_size {
        write_f32(s, o.field("s"));
    }
    if let Some(fc) = &d.fill_color {
        write_f32_slice(fc, o.field("fc"));
    }
    if let Some(sc) = &d.stroke_color {
        write_f32_slice(sc, o.field("sc"));
    }
    if let Some(sw) = d.stroke_width {
        write_f32(sw, o.field("sw"));
    }
    if let Some(of) = d.stroke_over_fill {
        o.field("of").push_str(if of { "true" } else { "false" });
    }
    if let Some(lh) = d.line_height {
        write_f32(lh, o.field("lh"));
    }
    if let Some(tr) = d.tracking {
        write_f32(tr, o.field("tr"));
    }
    if let Some(j) = d.justify {
        let _ = write!(o.field("j"), "{j}");
    }
    if let Some(ca) = d.text_caps {
        let _ = write!(o.field("ca"), "{ca}");
    }
    if let Some(ls) = d.baseline_shift {
        write_f32(ls, o.field("ls"));
    }
    if let Some(sz) = d.wrap_size {
        write_f32_slice(&sz, o.field("sz"));
    }
    if let Some(ps) = d.wrap_position {
        write_f32_slice(&ps, o.field("ps"));
    }
    o.finish();
}

pub(crate) fn text_slot_from_json(v: &Value) -> Option<TextSlot> {
    Some(TextSlot {
        keyframes: array_of(v.get("k")?, |kf| {
            Some(TextKeyframe {
                frame: kf.u32_field("t")?,
                text_document: text_document_from_json(kf.get("s")?)?,
            })
        })?,
        expression: v.opt_str_field("x")?,
    })
}

pub(crate) fn write_text_slot(t: &TextSlot, out: &mut String) {
    let mut o = ObjWriter::new(out);
    write_seq(o.field("k"), &t.keyframes, |kf, out| {
        let mut kfo = ObjWriter::new(out);
        let _ = write!(kfo.field("t"), "{}", kf.frame);
        write_text_document(&kf.text_document, kfo.field("s"));
        kfo.finish();
    });
    if let Some(x) = &t.expression {
        write_str(x, o.field("x"));
    }
    o.finish();
}
