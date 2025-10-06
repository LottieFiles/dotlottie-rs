use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
pub enum BindingsParserError {
    ParseError(String),
}
impl BindingsParserError {
    pub(crate) fn to_string(&self) -> String {
        match self {
            BindingsParserError::ParseError(msg) => format!("Parse error: {}", msg),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Bindings {
    pub bindings: HashMap<String, Binding>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Binding {
    #[serde(flatten)]
    pub r#type: BindingValue,
    // Disabled for stage 1
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub source: Option<BindingSource>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum BindingValue {
    Color { value: [f64; 3] },
    Vector { value: [f64; 2] },
    Scalar { value: f64 },
    Boolean { value: bool },
    Gradient { value: Vec<[f64; 4]> },
    Image { value: ImageValue },
    Text { value: String },
}

impl BindingValue {
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            BindingValue::Color { value } => serde_json::to_value(value),
            BindingValue::Vector { value } => serde_json::to_value(value),
            BindingValue::Scalar { value } => serde_json::to_value(value),
            BindingValue::Boolean { value } => serde_json::to_value(value),
            BindingValue::Gradient { value } => serde_json::to_value(value),
            BindingValue::Image { value } => serde_json::to_value(value),
            BindingValue::Text { value } => serde_json::to_value(value),
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

// todo
#[derive(Deserialize, Serialize, Debug)]
pub struct Converter {}

pub fn parse_bindings(json: &str) -> Result<Bindings, BindingsParserError> {
    serde_json::from_str(json).map_err(|e| BindingsParserError::ParseError(e.to_string()))
}
