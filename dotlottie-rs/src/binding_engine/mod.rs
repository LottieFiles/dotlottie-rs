use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::parser::{parse_bindings, BindingValue, Bindings, ImageValue};

pub mod parser;

#[derive(Debug)]
pub enum BindingsEngineError {
    ParseError(String),
    BindingNotFound(String),
    WrongBindingType {
        binding_name: String,
        expected: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct Theme {
    rules: Vec<Rule>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Rule {
    id: String,
    #[serde(rename = "type")]
    rule_type: String,
    value: Value,
}

pub struct BindingsEngineBuilder {
    bindings_definition: String,
    initial_dependencies: Option<HashMap<String, Vec<String>>>,
}

impl BindingsEngineBuilder {
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

    pub fn build(self) -> Result<BindingsEngine, BindingsEngineError> {
        let parsed_bindings = parse_bindings(&self.bindings_definition)
            .map_err(|e| BindingsEngineError::ParseError(e.to_string()))?;

        Ok(BindingsEngine {
            binding_container: parsed_bindings,
            theme_dependencies: self.initial_dependencies.unwrap_or_else(HashMap::new),
            was_updated: false,
        })
    }
}

pub struct BindingsEngine {
    binding_container: Bindings,

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
            binding_name: &str,
        ) -> Result<$return_type, BindingsEngineError> {
            self.binding_container
                .bindings
                .get(binding_name)
                .ok_or_else(|| BindingsEngineError::BindingNotFound(binding_name.to_string()))
                .and_then(|binding| match &binding.r#type {
                    BindingValue::$variant { value } => Ok(*value),
                    _ => Err(BindingsEngineError::WrongBindingType {
                        binding_name: binding_name.to_string(),
                        expected: $type_name.to_string(),
                    }),
                })
        }
    };
    // For Clone types (need to clone)
    ($method_name:ident, $variant:ident, $return_type:ty, $type_name:expr) => {
        pub fn $method_name(
            &self,
            binding_name: &str,
        ) -> Result<$return_type, BindingsEngineError> {
            self.binding_container
                .bindings
                .get(binding_name)
                .ok_or_else(|| BindingsEngineError::BindingNotFound(binding_name.to_string()))
                .and_then(|binding| match &binding.r#type {
                    BindingValue::$variant { value } => Ok(value.clone()),
                    _ => Err(BindingsEngineError::WrongBindingType {
                        binding_name: binding_name.to_string(),
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
            binding_name: &str,
            new_value: $param_type,
        ) -> Result<(), BindingsEngineError> {
            if let Some(binding) = self.binding_container.bindings.get_mut(binding_name) {
                match &mut binding.r#type {
                    BindingValue::$variant { value } => {
                        *value = new_value;

                        if self.theme_dependencies.contains_key(binding_name) {
                            println!(
                                ">> Binding key: {} modified, is linked to {:?} themes",
                                binding_name,
                                self.theme_dependencies.get(binding_name)
                            );
                            self.was_updated = true;
                        }

                        println!("[Bindings] Updated: {} to {:?}", binding_name, *value);
                        Ok(())
                    }
                    _ => Err(BindingsEngineError::WrongBindingType {
                        binding_name: binding_name.to_string(),
                        expected: $type_name.to_string(),
                    }),
                }
            } else {
                Err(BindingsEngineError::BindingNotFound(
                    binding_name.to_string(),
                ))
            }
        }
    };
    ($method_name:ident, $variant:ident, $param_type:ty, $type_name:expr) => {
        pub fn $method_name(
            &mut self,
            binding_name: &str,
            new_value: $param_type,
        ) -> Result<(), BindingsEngineError> {
            if let Some(binding) = self.binding_container.bindings.get_mut(binding_name) {
                match &mut binding.r#type {
                    BindingValue::$variant { value } => {
                        *value = new_value.clone();

                        if self.theme_dependencies.contains_key(binding_name) {
                            println!(
                                ">> Binding key: {} modified, is linked to {:?} themes",
                                binding_name,
                                self.theme_dependencies.get(binding_name)
                            );
                            self.was_updated = true;
                        }

                        println!("[Bindings] Updated: {} to {:?}", binding_name, *value);
                        Ok(())
                    }
                    _ => Err(BindingsEngineError::WrongBindingType {
                        binding_name: binding_name.to_string(),
                        expected: $type_name.to_string(),
                    }),
                }
            } else {
                Err(BindingsEngineError::BindingNotFound(
                    binding_name.to_string(),
                ))
            }
        }
    };
    // For &str -> String conversion (text binding)
    ($method_name:ident, $variant:ident, $param_type:ty, $type_name:expr, to_string) => {
        pub fn $method_name(
            &mut self,
            binding_name: &str,
            new_value: $param_type,
        ) -> Result<(), BindingsEngineError> {
            if let Some(binding) = self.binding_container.bindings.get_mut(binding_name) {
                match &mut binding.r#type {
                    BindingValue::$variant { value } => {
                        *value = new_value.to_string();

                        if self.theme_dependencies.contains_key(binding_name) {
                            println!(
                                ">> Binding key: {} modified, is linked to {:?} themes",
                                binding_name,
                                self.theme_dependencies.get(binding_name)
                            );
                            self.was_updated = true;
                        }

                        println!("[Bindings] Updated: {} to {}", binding_name, *value);
                        Ok(())
                    }
                    _ => Err(BindingsEngineError::WrongBindingType {
                        binding_name: binding_name.to_string(),
                        expected: $type_name.to_string(),
                    }),
                }
            } else {
                Err(BindingsEngineError::BindingNotFound(
                    binding_name.to_string(),
                ))
            }
        }
    };
}

impl BindingsEngine {
    pub fn builder(bindings_definition: &str) -> BindingsEngineBuilder {
        BindingsEngineBuilder::new(bindings_definition)
    }

    pub fn new(bindings_definition: &str) -> Result<BindingsEngine, BindingsEngineError> {
        let parsed_bindings = parse_bindings(bindings_definition)
            .map_err(|e| BindingsEngineError::ParseError(e.to_string()))?;

        Ok(BindingsEngine {
            binding_container: parsed_bindings,
            theme_dependencies: HashMap::new(),
            was_updated: false,
        })
    }

    // ============================================================================
    // Getters
    // ============================================================================

    impl_getter!(get_color_binding, Color, [f64; 3], "Color", copy);
    impl_getter!(get_vector_binding, Vector, [f64; 2], "Vector", copy);
    impl_getter!(get_scalar_binding, Scalar, f64, "Scalar", copy);
    impl_getter!(get_boolean_binding, Boolean, bool, "Boolean", copy);
    impl_getter!(get_gradient_binding, Gradient, Vec<[f64; 4]>, "Gradient");
    impl_getter!(get_image_binding, Image, ImageValue, "Image");
    impl_getter!(get_text_binding, Text, String, "Text");

    // ============================================================================
    // Mutators
    // ============================================================================

    impl_mutator!(mutate_color_binding, Color, [f64; 3], "Color", copy);
    impl_mutator!(mutate_vector_binding, Vector, [f64; 2], "Vector", copy);
    impl_mutator!(mutate_scalar_binding, Scalar, f64, "Scalar", copy);
    impl_mutator!(mutate_boolean_binding, Boolean, bool, "Boolean", copy);
    impl_mutator!(
        mutate_gradient_binding,
        Gradient,
        &Vec<[f64; 4]>,
        "Gradient"
    );
    impl_mutator!(mutate_image_binding, Image, &ImageValue, "Image");
    impl_mutator!(mutate_text_binding, Text, &str, "Text", to_string);

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

    fn replace_references(&mut self, theme: &mut Theme, theme_id: Option<&str>) {
        for rule in &mut theme.rules {
            let binding_id = if let Value::String(s) = &rule.value {
                s.strip_prefix('@').map(|stripped| stripped.to_string())
            } else {
                None
            };

            if let Some(binding_id) = binding_id {
                if let Some(binding) = self.binding_container.bindings.get(&binding_id) {
                    rule.value = binding.r#type.to_json_value();
                    println!(">> Binding value: {:?}", rule.value);

                    if let Some(theme_id) = theme_id {
                        self.add_new_theme_dependancy(theme_id, &binding_id);
                    }
                }
            }
        }
    }

    pub fn update_theme(
        &mut self,
        theme_data: &str,
        theme_id: Option<&str>,
    ) -> Result<String, BindingsEngineError> {
        let mut theme: Theme = serde_json::from_str(theme_data)
            .map_err(|e| BindingsEngineError::ParseError(format!("Theme parse error: {}", e)))?;

        self.replace_references(&mut theme, theme_id);

        serde_json::to_string(&theme)
            .map_err(|e| BindingsEngineError::ParseError(format!("Theme serialize error: {}", e)))
    }
}
