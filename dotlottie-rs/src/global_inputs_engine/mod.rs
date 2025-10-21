use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::parser::{parse_global_inputs, GlobalInputs, GradientStop};
use crate::parser::{GlobalInputValue, ImageValue};
pub mod parser;

#[derive(Debug)]
pub enum GlobalInputsEngineError {
    ParseError(String),
    BindingNotFound(String),
    WrongGlobalInputType {
        global_input_name: String,
        expected: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct Theme {
    rules: Vec<Rule>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ImageRule {
    id: Option<String>,
    width: Option<f64>,
    height: Option<f64>,
    url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<Vec<f64>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<Vec<f64>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_over_fill: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify: Option<Justify>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_caps: Option<TextCaps>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_shift: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_size: Option<Vec<f64>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_position: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Justify {
    Left,
    Right,
    Center,
    JustifyLastLeft,
    JustifyLastRight,
    JustifyLastCenter,
    JustifyLastFull,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextCaps {
    Regular,
    AllCaps,
    SmallCaps,
}

#[derive(Debug, Deserialize, Serialize)]
struct Rule {
    id: String,
    #[serde(rename = "type")]
    rule_type: String,
    value: Value,
}

pub struct GlobalInputsEngineBuilder {
    bindings_definition: String,
    initial_dependencies: Option<HashMap<String, Vec<String>>>,
}

impl GlobalInputsEngineBuilder {
    /// Create a new builder with the bindings definition
    pub fn new(bindings_definition: &str) -> Self {
        Self {
            bindings_definition: bindings_definition.to_string(),
            initial_dependencies: None,
        }
    }

    pub fn with_dependencies(mut self, dependencies: HashMap<String, Vec<String>>) -> Self {
        self.initial_dependencies = Some(dependencies);
        self
    }

    pub fn build(self) -> Result<GlobalInputsEngine, GlobalInputsEngineError> {
        let parsed_bindings = parse_global_inputs(&self.bindings_definition)
            .map_err(|e| GlobalInputsEngineError::ParseError(e.to_string()))?;

        println!("Found: {:?}", parsed_bindings);

        Ok(GlobalInputsEngine {
            global_inputs_container: parsed_bindings,
            theme_dependencies: self.initial_dependencies.unwrap_or_else(HashMap::new),
            was_updated: false,
        })
    }
}

pub struct GlobalInputsEngine {
    global_inputs_container: GlobalInputs,

    /**
     * Map<BindingId, [ThemeIds]>
     */
    theme_dependencies: HashMap<String, Vec<String>>,

    was_updated: bool,
}

// ============================================================================
// Macros for reducing boilerplate
// ============================================================================

macro_rules! impl_getter {
    // For Copy types (can use * instead of clone)
    ($method_name:ident, $variant:ident, $return_type:ty, $type_name:expr, copy) => {
        pub fn $method_name(
            &self,
            global_input_name: &str,
        ) -> Result<$return_type, GlobalInputsEngineError> {
            self.global_inputs_container
                .get(global_input_name)
                .ok_or_else(|| {
                    GlobalInputsEngineError::BindingNotFound(global_input_name.to_string())
                })
                .and_then(|binding| match &binding.r#type {
                    GlobalInputValue::$variant { value } => Ok(*value),
                    _ => Err(GlobalInputsEngineError::WrongGlobalInputType {
                        global_input_name: global_input_name.to_string(),
                        expected: $type_name.to_string(),
                    }),
                })
        }
    };
    // For Clone types (need to clone)
    ($method_name:ident, $variant:ident, $return_type:ty, $type_name:expr) => {
        pub fn $method_name(
            &self,
            global_input_name: &str,
        ) -> Result<$return_type, GlobalInputsEngineError> {
            self.global_inputs_container
                .get(global_input_name)
                .ok_or_else(|| {
                    GlobalInputsEngineError::BindingNotFound(global_input_name.to_string())
                })
                .and_then(|binding| match &binding.r#type {
                    GlobalInputValue::$variant { value } => Ok(value.clone()),
                    _ => Err(GlobalInputsEngineError::WrongGlobalInputType {
                        global_input_name: global_input_name.to_string(),
                        expected: $type_name.to_string(),
                    }),
                })
        }
    };
}

macro_rules! impl_mutator {
    ($method_name:ident, $variant:ident, $param_type:ty, $type_name:expr, copy) => {
        pub fn $method_name(
            &mut self,
            global_input_name: &str,
            new_value: $param_type,
        ) -> Result<(), GlobalInputsEngineError> {
            if let Some(binding) = self.global_inputs_container.get_mut(global_input_name) {
                match &mut binding.r#type {
                    GlobalInputValue::$variant { value } => {
                        *value = new_value;

                        if self.theme_dependencies.contains_key(global_input_name) {
                            println!(
                                ">> Binding key: {} modified, is linked to {:?} themes",
                                global_input_name,
                                self.theme_dependencies.get(global_input_name)
                            );
                        }

                        self.was_updated = true;
                        println!("[Bindings] Updated: {} to {:?}", global_input_name, *value);
                        Ok(())
                    }
                    _ => Err(GlobalInputsEngineError::WrongGlobalInputType {
                        global_input_name: global_input_name.to_string(),
                        expected: $type_name.to_string(),
                    }),
                }
            } else {
                Err(GlobalInputsEngineError::BindingNotFound(
                    global_input_name.to_string(),
                ))
            }
        }
    };
    ($method_name:ident, $variant:ident, $param_type:ty, $type_name:expr) => {
        pub fn $method_name(
            &mut self,
            global_input_name: &str,
            new_value: $param_type,
        ) -> Result<(), GlobalInputsEngineError> {
            if let Some(binding) = self.global_inputs_container.get_mut(global_input_name) {
                match &mut binding.r#type {
                    GlobalInputValue::$variant { value } => {
                        *value = new_value.clone();

                        if self.theme_dependencies.contains_key(global_input_name) {
                            println!(
                                ">> Binding key: {} modified, is linked to {:?} themes",
                                global_input_name,
                                self.theme_dependencies.get(global_input_name)
                            );
                        }
                        self.was_updated = true;

                        println!("[Bindings] Updated: {} to {:?}", global_input_name, *value);
                        Ok(())
                    }
                    _ => Err(GlobalInputsEngineError::WrongGlobalInputType {
                        global_input_name: global_input_name.to_string(),
                        expected: $type_name.to_string(),
                    }),
                }
            } else {
                Err(GlobalInputsEngineError::BindingNotFound(
                    global_input_name.to_string(),
                ))
            }
        }
    };
    // For &str -> String conversion (text binding)
    ($method_name:ident, $variant:ident, $param_type:ty, $type_name:expr, to_string) => {
        pub fn $method_name(
            &mut self,
            global_input_name: &str,
            new_value: $param_type,
        ) -> Result<(), GlobalInputsEngineError> {
            if let Some(binding) = self.global_inputs_container.get_mut(global_input_name) {
                match &mut binding.r#type {
                    GlobalInputValue::$variant { value } => {
                        *value = new_value.to_string();

                        if self.theme_dependencies.contains_key(global_input_name) {
                            println!(
                                ">> Binding key: {} modified, is linked to {:?} themes",
                                global_input_name,
                                self.theme_dependencies.get(global_input_name)
                            );
                        }
                        self.was_updated = true;

                        println!("[Bindings] Updated: {} to {}", global_input_name, *value);
                        Ok(())
                    }
                    _ => Err(GlobalInputsEngineError::WrongGlobalInputType {
                        global_input_name: global_input_name.to_string(),
                        expected: $type_name.to_string(),
                    }),
                }
            } else {
                Err(GlobalInputsEngineError::BindingNotFound(
                    global_input_name.to_string(),
                ))
            }
        }
    };
}

impl GlobalInputsEngine {
    pub fn builder(bindings_definition: &str) -> GlobalInputsEngineBuilder {
        GlobalInputsEngineBuilder::new(bindings_definition)
    }

    pub fn new(bindings_definition: &str) -> Result<GlobalInputsEngine, GlobalInputsEngineError> {
        let parsed_bindings = parse_global_inputs(bindings_definition)
            .map_err(|e| GlobalInputsEngineError::ParseError(e.to_string()))?;

        Ok(GlobalInputsEngine {
            global_inputs_container: parsed_bindings,
            theme_dependencies: HashMap::new(),
            was_updated: false,
        })
    }

    // ============================================================================
    // Getters
    // ============================================================================

    impl_getter!(global_inputs_get_color, Color, [f64; 3], "Color", copy);
    impl_getter!(global_inputs_get_vector, Vector, [f64; 2], "Vector", copy);
    impl_getter!(global_inputs_get_scalar, Scalar, f64, "Scalar", copy);
    impl_getter!(global_inputs_get_boolean, Boolean, bool, "Boolean", copy);
    impl_getter!(
        global_inputs_get_gradient,
        Gradient,
        Vec<GradientStop>,
        "Gradient"
    );
    impl_getter!(global_inputs_get_image, Image, ImageValue, "Image");
    impl_getter!(global_inputs_get_text, Text, String, "Text");

    // ============================================================================
    // Mutators
    // ============================================================================

    impl_mutator!(global_inputs_set_color, Color, [f64; 3], "Color", copy);
    impl_mutator!(global_inputs_set_vector, Vector, [f64; 2], "Vector", copy);
    impl_mutator!(global_inputs_set_scalar, Scalar, f64, "Scalar", copy);
    impl_mutator!(global_inputs_set_boolean, Boolean, bool, "Boolean", copy);
    impl_mutator!(
        global_inputs_set_gradient,
        Gradient,
        &Vec<GradientStop>,
        "Gradient"
    );
    impl_mutator!(global_inputs_set_image, Image, &ImageValue, "Image");
    impl_mutator!(global_inputs_set_string, Text, &str, "Text", to_string);

    // ============================================================================
    // Utility Methods
    // ============================================================================
    pub fn read_task_queue(&mut self) -> bool {
        let was_updated = self.was_updated;
        self.was_updated = false;
        return was_updated;
    }

    fn add_new_theme_dependancy(&mut self, theme_id: &str, binding_id: &str) {
        let items = self
            .theme_dependencies
            .entry(binding_id.to_string())
            .or_insert_with(Vec::new);

        if !items.contains(&theme_id.to_string()) {
            println!(">> Added theme dep: {} <> {}", binding_id, theme_id);
            items.push(theme_id.to_string());
        }
    }

    fn replace_references(
        &mut self,
        theme: &mut Theme,
        theme_id: Option<&str>,
    ) -> Result<(), GlobalInputsEngineError> {
        for rule in &mut theme.rules {
            let binding_id = if let Value::String(s) = &rule.value {
                s.strip_prefix('@').map(|stripped| stripped.to_string())
            } else if let Value::Object(s) = &rule.value {
                // The value is an object
                // This is the case for text and images
                // Try to parse as ImageRule, then extract reference if the ImageRule is a reference type.
                if let Ok(image_rule) =
                    serde_json::from_value::<ImageRule>(Value::Object(s.clone()))
                {
                    if let Some(id) = image_rule.id {
                        id.strip_prefix('@').map(|reference| reference.to_string())
                    } else {
                        return Err(GlobalInputsEngineError::ParseError(
                            "Failed to replace references".to_string(),
                        ));
                    }
                } else if let Ok(text_rule) =
                    // todo!!
                    serde_json::from_value::<TextRule>(Value::Object(s.clone()))
                {
                    return Err(GlobalInputsEngineError::ParseError(
                        "Failed to replace references".to_string(),
                    ));
                } else {
                    return Err(GlobalInputsEngineError::ParseError(
                        "Failed to replace references".to_string(),
                    ));
                }
            } else {
                return Err(GlobalInputsEngineError::ParseError(
                    "Failed to replace references".to_string(),
                ));
            };

            if let Some(binding_id) = binding_id {
                if let Some(binding) = self.global_inputs_container.get(&binding_id) {
                    rule.value = binding.r#type.to_json_value();
                    println!(">> Binding value: {:?}", rule.value);

                    if let Some(theme_id) = theme_id {
                        self.add_new_theme_dependancy(theme_id, &binding_id);
                    }

                    return Ok(());
                }
            } else {
                println!("binding not found");
                return Err(GlobalInputsEngineError::BindingNotFound("".to_string()));
            }
        }
        Ok(())
    }

    pub fn update_theme(
        &mut self,
        theme_data: &str,
        theme_id: Option<&str>,
    ) -> Result<String, GlobalInputsEngineError> {
        let mut theme: Theme = serde_json::from_str(theme_data).map_err(|e| {
            GlobalInputsEngineError::ParseError(format!("Theme parse error: {}", e))
        })?;

        self.replace_references(&mut theme, theme_id);

        serde_json::to_string(&theme).map_err(|e| {
            GlobalInputsEngineError::ParseError(format!("Theme serialize error: {}", e))
        })
    }
}
