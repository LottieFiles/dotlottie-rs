use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
pub enum GlobalInputsParserError {
    ParseError(String),
}
impl GlobalInputsParserError {
    pub(crate) fn to_string(&self) -> String {
        match self {
            GlobalInputsParserError::ParseError(msg) => format!("Parse error: {}", msg),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GlobalInput {
    #[serde(flatten)]
    pub r#type: GlobalInputValue,
    // Disabled for stage 1
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub source: Option<BindingSource>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GradientStop {
    pub color: Vec<f64>,
    pub offset: f64,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum GlobalInputValue {
    Color { value: [f64; 3] },
    Vector { value: [f64; 2] },
    Scalar { value: f64 },
    Boolean { value: bool },
    Gradient { value: Vec<GradientStop> },
    Image { value: ImageValue },
    Text { value: String },
}

impl GlobalInputValue {
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            GlobalInputValue::Color { value } => serde_json::to_value(value),
            GlobalInputValue::Vector { value } => serde_json::to_value(value),
            GlobalInputValue::Scalar { value } => serde_json::to_value(value),
            GlobalInputValue::Boolean { value } => serde_json::to_value(value),
            GlobalInputValue::Gradient { value } => serde_json::to_value(value),
            GlobalInputValue::Image { value } => serde_json::to_value(value),
            GlobalInputValue::Text { value } => serde_json::to_value(value),
        }
        .unwrap_or(serde_json::Value::Null)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ImageValue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "PascalCase")]
pub enum BindingSource {
    Mapped {
        #[serde(rename = "bindFrom")]
        bind_from: String,
        #[serde(rename = "bindMethod")]
        bind_method: BindMethod,
        converters: Vec<Converter>,
    },
    Property {
        scope: PropertyScope,
        #[serde(flatten)]
        property_type: PropertyType,
        #[serde(default = "default_update_frequency")]
        #[serde(rename = "updateFrequency")]
        update_frequency: UpdateFrequency,
        converters: Vec<Converter>,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub enum BindMethod {
    Sync,
    Once,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum PropertyScope {
    Animation,
    Layer,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum PropertyType {
    Animation {
        property: AnimationProperty,
    },
    Layer {
        #[serde(rename = "layerName")]
        layer_name: String,
        property: LayerProperty,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AnimationProperty {
    CurrentFrame,
    LoopCount,
    Speed,
    Direction,
    CanvasSize,
    BackgrondColor,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum LayerProperty {
    FillColor,
    BorderColor,
    Position,
    Scale,
    Rotation,
    Opacity,
    Text,
    StrokeColor,
    StrokeWidth,
    Anchor,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum UpdateFrequency {
    OnChange,
    OnTick,
}

fn default_update_frequency() -> UpdateFrequency {
    UpdateFrequency::OnChange
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Converter {}

pub type GlobalInputs = HashMap<String, GlobalInput>;

pub fn parse_global_inputs(json: &str) -> Result<GlobalInputs, GlobalInputsParserError> {
    serde_json::from_str(json).map_err(|e| GlobalInputsParserError::ParseError(e.to_string()))
}
