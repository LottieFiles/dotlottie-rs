use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{parser::binding_path::BindingPath, GradientStop, ImageValue};

pub mod binding_path;
pub mod boolean_path;
pub mod color_path;
pub mod gradient_path;
pub mod numeric_path;
pub mod string_path;
pub mod vector_path;

#[derive(Debug)]
pub enum GlobalInputsParserError {
    ParseError(String),
}

impl std::fmt::Display for GlobalInputsParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlobalInputsParserError::ParseError(msg) => write!(f, "Parse error: {msg}"),
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

#[derive(Debug, Clone)]
pub struct ResolvedThemeBinding {
    pub rule_id: String,
    pub theme_id: String,
    pub path: BindingPath,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GlobalInput {
    #[serde(flatten)]
    pub r#type: GlobalInputValue,
    pub bindings: Bindings,

    #[serde(skip)]
    pub resolved_theme_bindings: Vec<ResolvedThemeBinding>,
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

pub type GlobalInputs = HashMap<String, GlobalInput>;

pub fn parse_global_inputs(json: &str) -> Result<GlobalInputs, GlobalInputsParserError> {
    serde_json::from_str(json).map_err(|e| GlobalInputsParserError::ParseError(e.to_string()))
}
