use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{GradientStop, ImageValue, ResolvedThemeBinding};

pub mod color_path;

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
#[serde(rename_all = "camelCase")]
pub struct ThemeBinding {
    pub theme_id: String,
    pub rule_id: String,
    pub path: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StateMachineBinding {
    pub state_machine_id: String,
    pub input_name: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bindings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub themes: Option<Vec<ThemeBinding>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_machines: Option<Vec<StateMachineBinding>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GlobalInput {
    #[serde(flatten)]
    pub r#type: GlobalInputValue,
    pub bindings: Bindings,

    #[serde(skip)]
    pub resolved_theme_bindings: Vec<ResolvedThemeBinding>,
    // Disabled for stage 1
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub source: Option<BindingSource>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum GlobalInputValue {
    Color { value: Vec<f32> },
    Vector { value: [f32; 2] },
    Numeric { value: f32 },
    Boolean { value: bool },
    Gradient { value: Vec<GradientStop> },
    Image { value: ImageValue },
    String { value: String },
}

impl GlobalInputValue {
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            GlobalInputValue::Color { value } => serde_json::to_value(value),
            GlobalInputValue::Vector { value } => serde_json::to_value(value),
            GlobalInputValue::Numeric { value } => serde_json::to_value(value),
            GlobalInputValue::Boolean { value } => serde_json::to_value(value),
            GlobalInputValue::Gradient { value } => serde_json::to_value(value),
            GlobalInputValue::Image { value } => serde_json::to_value(value),
            GlobalInputValue::String { value } => serde_json::to_value(value),
        }
        .unwrap_or(serde_json::Value::Null)
    }
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
