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
use crate::{GradientStop, ImageValue, LottieRenderer, StateMachineEngine};
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
}

impl GlobalInputsEngineBuilder {
    pub fn new(bindings_definition: &str) -> Self {
        Self {
            bindings_definition: bindings_definition.to_string(),
        }
    }

    pub fn build(self) -> Result<GlobalInputsEngine, GlobalInputsEngineError> {
        let parsed_bindings = parse_global_inputs(&self.bindings_definition)
            .map_err(|e| GlobalInputsEngineError::ParseError(e.to_string()))?;

        Ok(GlobalInputsEngine {
            global_inputs_container: parsed_bindings,
            state_machine_binding_cache: HashMap::new(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StateMachineBindingKey {
    binding_id: String,
    state_machine_id: String,
}

impl StateMachineBindingKey {
    pub fn new(binding_id: &str, state_machine_id: &str) -> Self {
        Self {
            binding_id: binding_id.to_string(),
            state_machine_id: state_machine_id.to_string(),
        }
    }
}

pub struct GlobalInputsEngine {
    global_inputs_container: GlobalInputs,
    state_machine_binding_cache: HashMap<StateMachineBindingKey, Vec<String>>,
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

impl GlobalInputsEngine {
    pub fn builder(bindings_definition: &str) -> GlobalInputsEngineBuilder {
        GlobalInputsEngineBuilder::new(bindings_definition)
    }

    pub fn new(bindings_definition: &str) -> Result<GlobalInputsEngine, GlobalInputsEngineError> {
        let parsed_bindings = parse_global_inputs(bindings_definition)
            .map_err(|e| GlobalInputsEngineError::ParseError(e.to_string()))?;

        Ok(GlobalInputsEngine {
            global_inputs_container: parsed_bindings,
            state_machine_binding_cache: HashMap::new(),
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
    // Utility Methods
    // ============================================================================
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
        new_value: &Vec<f32>,
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

    pub fn apply_to_state_machine(
        &mut self,
        binding_id: &str,
        state_machine_engine: &mut StateMachineEngine,
    ) -> bool {
        let cache_key = StateMachineBindingKey::new(binding_id, &state_machine_engine.id);

        // Check cache first
        let input_names = if let Some(cached) = self.state_machine_binding_cache.get(&cache_key) {
            cached.clone()
        } else {
            // Resolve and cache
            let input_names = self.resolve_input_names(binding_id, &state_machine_engine.id);
            self.state_machine_binding_cache
                .insert(cache_key, input_names.clone());
            input_names
        };

        if input_names.is_empty() {
            return false;
        }

        let Some(global_input) = self.global_inputs_container.get(binding_id) else {
            return false;
        };

        let mut success = false;
        for input_name in &input_names {
            let result = match &global_input.r#type {
                GlobalInputValue::Numeric { value } => state_machine_engine
                    .set_numeric_input(input_name, *value, true, false)
                    .is_some(),
                GlobalInputValue::Boolean { value } => state_machine_engine
                    .set_boolean_input(input_name, *value, true, false)
                    .is_some(),
                GlobalInputValue::String { value } => state_machine_engine
                    .set_string_input(input_name, value, true, false)
                    .is_some(),
                _ => false,
            };
            success = success || result;
        }

        success
    }

    fn resolve_input_names(&self, binding_id: &str, state_machine_id: &str) -> Vec<String> {
        let Some(global_input) = self.global_inputs_container.get(binding_id) else {
            return vec![];
        };

        let Some(state_machine_bindings) = &global_input.bindings.state_machines else {
            return vec![];
        };

        for sm_binding in state_machine_bindings {
            if sm_binding.state_machine_id == state_machine_id {
                return sm_binding.input_name.clone();
            }
        }

        vec![]
    }

    pub fn apply_to_slots(
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
                                // missing image
                                _ => {
                                    continue;
                                }
                            };

                        let r = binding_path.apply(renderer, &theme_binding.rule_id, binding_value);
                        if r.is_err() {
                            GlobalInputsEngineError::ParseError(
                                "Failed to apply global inputs on to current slots.".to_string(),
                            );
                        }

                        //todo: This doesnt detect duplicates
                        global_input
                            .resolved_theme_bindings
                            .push(ResolvedThemeBinding {
                                rule_id: theme_binding.rule_id.clone(),
                                theme_id: theme_binding.theme_id.clone(),
                                path: binding_path,
                            });

                        //todo: This doesnt clear out ever
                        let _ = renderer.apply_all_slots();
                    }
                }
            }
        }
        Ok("".to_string())
    }

    pub fn clear_resolved_theme_bindings(&mut self) {
        for (_, global_input) in self.global_inputs_container.iter_mut() {
            global_input.resolved_theme_bindings.clear();
        }
    }
}
