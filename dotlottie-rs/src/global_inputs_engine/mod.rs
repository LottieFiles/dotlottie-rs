use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::parser::color_path::ColorPath;
use crate::parser::GlobalInputValue;
use crate::parser::{parse_global_inputs, GlobalInputs};
use crate::{GradientStop, ImageValue, LottieRenderer};
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

impl From<String> for GlobalInputsEngineError {
    fn from(s: String) -> Self {
        GlobalInputsEngineError::ParseError(s)
    }
}

#[derive(Debug)]
pub enum BindingUsageSlotType {
    Color,
    Gradient,
    Image,
    String,
    Numeric,
    Vector,
}

#[derive(Debug, Default)]
pub struct ResolvedThemeBinding {
    pub rule_id: String,
    pub theme_id: String,
    pub path: ColorPath,
}

#[derive(Debug, Deserialize, Serialize)]
struct Rule {
    id: String,
    #[serde(rename = "type")]
    rule_type: String,
    value: Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct Theme {
    rules: Vec<Rule>,
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

        // println!("Found: {:?}", parsed_bindings);
        // for (key, value) in &parsed_bindings {
        //     match &value.r#type {
        //         GlobalInputValue::Color { value } => println!("Color: {:?}", value),
        //         GlobalInputValue::Vector { value } => println!("Vector: {:?}", value),
        //         GlobalInputValue::Numeric { value } => println!("Numeric: {}", value),
        //         GlobalInputValue::Boolean { value } => println!("Boolean: {}", value),
        //         GlobalInputValue::Gradient { value } => println!("Gradient: {:?}", value),
        //         GlobalInputValue::Image { value } => println!("Image: {:?}", value),
        //         GlobalInputValue::String { value } => println!("String: {}", value),
        //     }
        // }

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

                        if self.theme_dependencies.contains_key(global_input_name) {}

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

                        if self.theme_dependencies.contains_key(global_input_name) {}
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
    // For &str -> String conversion (String binding)
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

                        if self.theme_dependencies.contains_key(global_input_name) {}
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

    // impl_getter!(global_inputs_get_color, Color, vec<f, "Color", copy);
    impl_getter!(global_inputs_get_vector, Vector, [f32; 2], "Vector", copy);
    impl_getter!(global_inputs_get_numeric, Numeric, f32, "Numeric", copy);
    impl_getter!(global_inputs_get_boolean, Boolean, bool, "Boolean", copy);
    impl_getter!(
        global_inputs_get_gradient,
        Gradient,
        Vec<GradientStop>,
        "Gradient"
    );
    impl_getter!(global_inputs_get_image, Image, ImageValue, "Image");
    impl_getter!(global_inputs_get_string, String, String, "String");

    // ============================================================================
    // Mutators
    // ============================================================================

    // impl_mutator!(global_inputs_set_color, Color, [f32; 3], "Color", copy);
    impl_mutator!(global_inputs_set_vector, Vector, [f32; 2], "Vector", copy);
    impl_mutator!(global_inputs_set_numeric, Numeric, f32, "Numeric", copy);
    impl_mutator!(global_inputs_set_boolean, Boolean, bool, "Boolean", copy);
    impl_mutator!(
        global_inputs_set_gradient,
        Gradient,
        &Vec<GradientStop>,
        "Gradient"
    );
    impl_mutator!(global_inputs_set_image, Image, &ImageValue, "Image");
    impl_mutator!(global_inputs_set_string, String, &str, "String", to_string);

    // ============================================================================
    // Utility Methods
    // ============================================================================
    pub fn read_task_queue(&mut self) -> bool {
        let was_updated = self.was_updated;
        self.was_updated = false;
        return was_updated;
    }

    fn vec_tof32(&self, input: &Vec<f32>) -> [f32; 4] {
        let rgba_value = if input.len() >= 4 {
            [input[0], input[1], input[2], input[3]]
        } else if input.len() >= 3 {
            [input[0], input[1], input[2], 1.0]
        } else {
            [0.0, 0.0, 0.0, 1.0]
        };

        rgba_value
    }

    pub fn global_inputs_get_color(&self, binding_name: &str) -> Option<[f32; 4]> {
        let color: Option<&parser::GlobalInput> = self.global_inputs_container.get(binding_name);

        if let Some(color) = color {
            match &color.r#type {
                GlobalInputValue::Color { value } => {
                    if value.len() < 3 {
                        None
                    } else {
                        let rgba_value = self.vec_tof32(value);

                        Some(rgba_value)
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn global_inputs_set_color(
        &mut self,
        global_input_name: &str,
        new_value: [f32; 4],
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> bool {
        println!(
            ">>> global_inputs_set_color called for: {}",
            global_input_name
        );

        if let Some(binding) = self.global_inputs_container.get_mut(global_input_name) {
            println!(
                ">>> Found binding, resolved_theme_bindings count: {}",
                binding.resolved_theme_bindings.len()
            );

            match &mut binding.r#type {
                GlobalInputValue::Color { value } => {
                    *value = new_value.to_vec();

                    for resolved in &binding.resolved_theme_bindings {
                        if resolved.path.targets_gradient() {
                            if let Some(gradient_slot) =
                                renderer.get_gradient_slot(&resolved.rule_id)
                            {
                                if let Err(e) = resolved
                                    .path
                                    .apply_to_gradient(gradient_slot, &new_value.to_vec())
                                {
                                    eprintln!("Failed to apply gradient path: {e}");
                                }
                            }
                        } else {
                            if let Some(color_slot) = renderer.get_color_slot(&resolved.rule_id) {
                                if let Err(e) = resolved
                                    .path
                                    .apply_to_color(color_slot, &new_value.to_vec())
                                {
                                    eprintln!("Failed to apply color path: {e}");
                                }
                            }
                        }
                    }
                    let _ = renderer.apply_all_slots();
                    return true;
                }
                _ => {}
            }
        } else {
            println!(">>> Binding not found: {}", global_input_name);
        }

        false
    }

    pub fn update_theme(
        &mut self,
        theme_id: &str,
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> Result<String, GlobalInputsEngineError> {
        for (_, global_input) in self.global_inputs_container.iter_mut() {
            if let Some(themes) = &global_input.bindings.themes {
                for theme_binding in themes {
                    if theme_binding.theme_id == theme_id
                        || theme_binding.theme_id == "*".to_string()
                    {
                        match &global_input.r#type {
                            GlobalInputValue::Color { value } => {
                                let parsed_path = ColorPath::parse(&theme_binding.path)?;

                                // Color input is going in to a Gradient
                                if parsed_path.targets_gradient() {
                                    if let Some(gradient_slot) =
                                        renderer.get_gradient_slot(&theme_binding.rule_id)
                                    {
                                        parsed_path.apply_to_gradient(gradient_slot, value)?;

                                        global_input.resolved_theme_bindings.push(
                                            ResolvedThemeBinding {
                                                rule_id: theme_binding.rule_id.clone(),
                                                theme_id: theme_binding.theme_id.clone(),
                                                path: parsed_path,
                                            },
                                        );

                                        let _ = renderer.apply_all_slots();
                                    }
                                } else {
                                    // Color input is going in to a Color
                                    if let Some(color_slot) =
                                        renderer.get_color_slot(&theme_binding.rule_id)
                                    {
                                        parsed_path.apply_to_color(color_slot, value)?;

                                        global_input.resolved_theme_bindings.push(
                                            ResolvedThemeBinding {
                                                rule_id: theme_binding.rule_id.clone(),
                                                theme_id: theme_binding.theme_id.clone(),
                                                path: parsed_path,
                                            },
                                        );

                                        let _ = renderer.apply_all_slots();
                                    }
                                }
                            }
                            GlobalInputValue::Vector { value } => {}
                            GlobalInputValue::Numeric { value } => {}
                            GlobalInputValue::Boolean { value } => {}
                            GlobalInputValue::Gradient { value } => {}
                            GlobalInputValue::Image { value } => {}
                            GlobalInputValue::String { value } => {}
                        }
                    }
                }
            }
        }
        Ok("".to_string())
    }
}
