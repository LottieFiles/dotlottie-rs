use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::parser::binding_path::{BindingPath, BindingValue};
use crate::parser::boolean_path::BooleanPath;
use crate::parser::color_path::ColorPath;
use crate::parser::gradient_path::GradientPath;
use crate::parser::numeric_path::NumericPath;
use crate::parser::string_path::StringPath;
use crate::parser::vector_path::VectorPath;
use crate::parser::{parse_global_inputs, GlobalInputs};
use crate::parser::{GlobalInputValue, ResolvedThemeBinding};
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
    // impl_mutator!(global_inputs_set_vector, Vector, [f32; 2], "Vector", copy);
    // impl_mutator!(global_inputs_set_numeric, Numeric, f32, "Numeric", copy);
    // impl_mutator!(global_inputs_set_boolean, Boolean, bool, "Boolean", copy);
    // impl_mutator!(
    //     global_inputs_set_gradient,
    //     Gradient,
    //     &Vec<GradientStop>,
    //     "Gradient"
    // );
    impl_mutator!(global_inputs_set_image, Image, &ImageValue, "Image");
    // impl_mutator!(global_inputs_set_string, String, &str, "String", to_string);

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
        if let Some(binding) = self.global_inputs_container.get_mut(global_input_name) {
            match &mut binding.r#type {
                GlobalInputValue::Color { value } => {
                    *value = new_value.to_vec();

                    for resolved in &binding.resolved_theme_bindings {
                        if let Err(e) = resolved.path.apply(
                            renderer,
                            &resolved.rule_id,
                            parser::binding_path::BindingValue::Vector(&new_value),
                        ) {
                            eprintln!("Failed to apply path: {e}");
                        }
                    }
                    let _ = renderer.apply_all_slots();
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    pub fn global_inputs_set_numeric(
        &mut self,
        global_input_name: &str,
        new_value: f32,
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> bool {
        if let Some(binding) = self.global_inputs_container.get_mut(global_input_name) {
            match &mut binding.r#type {
                GlobalInputValue::Numeric { value } => {
                    *value = new_value;

                    for resolved in &binding.resolved_theme_bindings {
                        if let Err(e) = resolved.path.apply(
                            renderer,
                            &resolved.rule_id,
                            parser::binding_path::BindingValue::Numeric(new_value),
                        ) {
                            eprintln!("Failed to apply path: {e}");
                        }
                    }
                    let _ = renderer.apply_all_slots();
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    pub fn global_inputs_set_string(
        &mut self,
        global_input_name: &str,
        new_value: &str,
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> bool {
        if let Some(binding) = self.global_inputs_container.get_mut(global_input_name) {
            match &mut binding.r#type {
                GlobalInputValue::String { value } => {
                    *value = new_value.to_string();

                    for resolved in &binding.resolved_theme_bindings {
                        if let Err(e) = resolved.path.apply(
                            renderer,
                            &resolved.rule_id,
                            parser::binding_path::BindingValue::String(new_value),
                        ) {
                            eprintln!("Failed to apply path: {e}");
                        }
                    }
                    let _ = renderer.apply_all_slots();
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    pub fn global_inputs_set_boolean(
        &mut self,
        global_input_name: &str,
        new_value: bool,
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> bool {
        if let Some(binding) = self.global_inputs_container.get_mut(global_input_name) {
            match &mut binding.r#type {
                GlobalInputValue::Boolean { value } => {
                    *value = new_value;

                    for resolved in &binding.resolved_theme_bindings {
                        if let Err(e) = resolved.path.apply(
                            renderer,
                            &resolved.rule_id,
                            parser::binding_path::BindingValue::Boolean(new_value),
                        ) {
                            eprintln!("Failed to apply path: {e}");
                        }
                    }
                    let _ = renderer.apply_all_slots();
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    pub fn global_inputs_set_vector(
        &mut self,
        global_input_name: &str,
        new_value: [f32; 2],
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> bool {
        if let Some(binding) = self.global_inputs_container.get_mut(global_input_name) {
            match &mut binding.r#type {
                GlobalInputValue::Vector { value } => {
                    *value = new_value;

                    for resolved in &binding.resolved_theme_bindings {
                        if let Err(e) = resolved.path.apply(
                            renderer,
                            &resolved.rule_id,
                            parser::binding_path::BindingValue::Vector(&new_value),
                        ) {
                            eprintln!("Failed to apply path: {e}");
                        }
                    }
                    let _ = renderer.apply_all_slots();
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    pub fn global_inputs_set_gradient(
        &mut self,
        global_input_name: &str,
        new_value: &Vec<GradientStop>,
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> bool {
        if let Some(binding) = self.global_inputs_container.get_mut(global_input_name) {
            match &mut binding.r#type {
                GlobalInputValue::Gradient { value } => {
                    *value = new_value.to_vec();

                    for resolved in &binding.resolved_theme_bindings {
                        if let Err(e) = resolved.path.apply(
                            renderer,
                            &resolved.rule_id,
                            BindingValue::Gradient(new_value.as_slice()),
                        ) {
                            eprintln!("Failed to apply path: {e}");
                        }
                    }
                    let _ = renderer.apply_all_slots();
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    pub fn insert_in_to_slots(
        &mut self,
        theme_id: &str,
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> Result<String, GlobalInputsEngineError> {
        for (_, global_input) in self.global_inputs_container.iter_mut() {
            if let Some(themes) = &global_input.bindings.themes {
                for theme_binding in themes {
                    if theme_binding.theme_id == theme_id || theme_binding.theme_id == "*" {
                        let (binding_path, binding_value): (BindingPath, BindingValue) =
                            match &global_input.r#type {
                                GlobalInputValue::Color { value } => {
                                    let parsed = ColorPath::parse(&theme_binding.path)?;
                                    (parsed.into(), BindingValue::Color(value.as_slice()))
                                }
                                GlobalInputValue::Vector { value } => {
                                    let parsed = VectorPath::parse(&theme_binding.path)?;
                                    (parsed.into(), BindingValue::Vector(value.as_slice()))
                                }
                                GlobalInputValue::Numeric { value } => {
                                    let parsed = NumericPath::parse(&theme_binding.path)?;
                                    (parsed.into(), BindingValue::Numeric(*value))
                                }
                                GlobalInputValue::String { value } => {
                                    let parsed = StringPath::parse(&theme_binding.path)?;
                                    (parsed.into(), BindingValue::String(value.as_str()))
                                }
                                GlobalInputValue::Boolean { value } => {
                                    let parsed = BooleanPath::parse(&theme_binding.path)?;
                                    (parsed.into(), BindingValue::Boolean(*value))
                                }
                                GlobalInputValue::Gradient { value } => {
                                    let parsed = GradientPath::parse(&theme_binding.path)?;
                                    (parsed.into(), BindingValue::Gradient(&value))
                                }
                                // Skip unimplemented types for now
                                _ => continue,
                            };

                        binding_path.apply(renderer, &theme_binding.rule_id, binding_value)?;

                        global_input
                            .resolved_theme_bindings
                            .push(ResolvedThemeBinding {
                                rule_id: theme_binding.rule_id.clone(),
                                theme_id: theme_binding.theme_id.clone(),
                                path: binding_path,
                            });

                        let _ = renderer.apply_all_slots();
                    }
                }
            }
        }
        Ok("".to_string())
    }
}
