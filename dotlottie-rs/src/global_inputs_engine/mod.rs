use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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

pub trait GlobalInputsObserver: Send + Sync {
    fn on_color_global_input_value_change(
        &self,
        global_input_name: String,
        old_value: Vec<f32>,
        new_value: Vec<f32>,
    );
    fn on_gradient_global_input_value_change(
        &self,
        global_input_name: String,
        old_value: Vec<f32>,
        new_value: Vec<f32>,
    );
    fn on_numeric_global_input_value_change(
        &self,
        global_input_name: String,
        old_value: f32,
        new_value: f32,
    );
    fn on_boolean_global_input_value_change(
        &self,
        global_input_name: String,
        old_value: bool,
        new_value: bool,
    );
    fn on_string_global_input_value_change(
        &self,
        global_input_name: String,
        old_value: String,
        new_value: String,
    );
    fn on_vector_global_input_value_change(
        &self,
        global_input_name: String,
        old_value: [f32; 2],
        new_value: [f32; 2],
    );
}

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
            observers: RwLock::new(Vec::new()),
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
    pub observers: RwLock<Vec<Arc<dyn GlobalInputsObserver>>>,
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
            observers: RwLock::new(Vec::new()),
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

    /// Converts Vec<GradientStop> to Vec<f32> for observer notifications
    /// Format: [offset, r, g, b, a, offset, r, g, b, a, ...]
    fn gradient_stops_to_vec(stops: &Vec<GradientStop>) -> Vec<f32> {
        let mut result = Vec::with_capacity(stops.len() * 5);
        for stop in stops {
            result.push(stop.offset);
            result.push(stop.color[0]);
            result.push(stop.color[1]);
            result.push(stop.color[2]);
            result.push(if stop.color.len() >= 4 {
                stop.color[3]
            } else {
                1.0
            });
        }
        result
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

    /// Helper to collect all state machine input names for a global input
    fn collect_state_machine_input_names(
        &self,
        global_input: &parser::GlobalInput,
    ) -> Option<Vec<String>> {
        let state_machine_bindings = global_input.bindings.state_machines.as_ref()?;

        let mut all_input_names = Vec::new();
        for sm_binding in state_machine_bindings {
            all_input_names.extend(sm_binding.input_name.clone());
        }

        if all_input_names.is_empty() {
            None
        } else {
            Some(all_input_names)
        }
    }

    pub fn global_inputs_set_boolean(
        &mut self,
        global_input_name: &str,
        new_value: bool,
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> bool {
        let Some(binding) = self.global_inputs_container.get_mut(global_input_name) else {
            return false;
        };

        let GlobalInputValue::Boolean { value } = &mut binding.r#type else {
            return false;
        };

        let old_value = *value;
        *value = new_value;

        for resolved in &binding.resolved_theme_bindings {
            if let Err(_) = resolved.path.apply(
                renderer,
                &resolved.rule_id,
                BindingValue::Boolean(new_value),
            ) {
                return false;
            }
        }
        let _ = renderer.apply_all_slots();

        self.observe_boolean_global_input_value_change(global_input_name, old_value, new_value);

        true
    }

    pub fn global_inputs_set_numeric(
        &mut self,
        global_input_name: &str,
        new_value: f32,
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> bool {
        let Some(binding) = self.global_inputs_container.get_mut(global_input_name) else {
            return false;
        };

        let GlobalInputValue::Numeric { value } = &mut binding.r#type else {
            return false;
        };

        let old_value = *value;
        *value = new_value;

        for resolved in &binding.resolved_theme_bindings {
            if let Err(_) = resolved.path.apply(
                renderer,
                &resolved.rule_id,
                BindingValue::Numeric(new_value),
            ) {
                return false;
            }
        }
        let _ = renderer.apply_all_slots();

        self.observe_numeric_global_input_value_change(global_input_name, old_value, new_value);

        true
    }

    pub fn global_inputs_set_string(
        &mut self,
        global_input_name: &str,
        new_value: &str,
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> bool {
        let Some(binding) = self.global_inputs_container.get_mut(global_input_name) else {
            return false;
        };

        let GlobalInputValue::String { value } = &mut binding.r#type else {
            return false;
        };

        let old_value = value.clone();
        *value = new_value.to_string();

        for resolved in &binding.resolved_theme_bindings {
            if let Err(_) =
                resolved
                    .path
                    .apply(renderer, &resolved.rule_id, BindingValue::String(new_value))
            {
                return false;
            }
        }
        let _ = renderer.apply_all_slots();

        self.observe_string_global_input_value_change(global_input_name, &old_value, new_value);

        true
    }

    pub fn global_inputs_set_color(
        &mut self,
        global_input_name: &str,
        new_value: &Vec<f32>,
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> bool {
        let Some(binding) = self.global_inputs_container.get_mut(global_input_name) else {
            return false;
        };

        let GlobalInputValue::Color { value } = &mut binding.r#type else {
            return false;
        };

        let old_value = value.clone();
        *value = new_value.clone();

        for resolved in &binding.resolved_theme_bindings {
            if let Err(_) = resolved.path.apply(
                renderer,
                &resolved.rule_id,
                BindingValue::Color(new_value.as_slice()),
            ) {
                return false;
            }
        }
        let _ = renderer.apply_all_slots();

        self.observe_color_global_input_value_change(global_input_name, &old_value, new_value);

        true
    }

    pub fn global_inputs_set_vector(
        &mut self,
        global_input_name: &str,
        new_value: [f32; 2],
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> bool {
        let Some(binding) = self.global_inputs_container.get_mut(global_input_name) else {
            return false;
        };

        let GlobalInputValue::Vector { value } = &mut binding.r#type else {
            return false;
        };

        let old_value = *value;
        *value = new_value;

        for resolved in &binding.resolved_theme_bindings {
            if let Err(_) = resolved.path.apply(
                renderer,
                &resolved.rule_id,
                BindingValue::Vector(&new_value),
            ) {
                return false;
            }
        }
        let _ = renderer.apply_all_slots();

        self.observe_vector_global_input_value_change(global_input_name, old_value, new_value);

        true
    }

    pub fn global_inputs_set_gradient(
        &mut self,
        global_input_name: &str,
        new_value: &Vec<GradientStop>,
        renderer: &mut Box<dyn LottieRenderer>,
    ) -> bool {
        let Some(binding) = self.global_inputs_container.get_mut(global_input_name) else {
            return false;
        };

        let GlobalInputValue::Gradient { value } = &mut binding.r#type else {
            return false;
        };

        let old_value = Self::gradient_stops_to_vec(value);
        *value = new_value.clone();
        let new_value_vec = Self::gradient_stops_to_vec(new_value);

        for resolved in &binding.resolved_theme_bindings {
            if let Err(_) = resolved.path.apply(
                renderer,
                &resolved.rule_id,
                BindingValue::Gradient(new_value.as_slice()),
            ) {
                return false;
            }
        }
        let _ = renderer.apply_all_slots();

        self.observe_gradient_global_input_value_change(
            global_input_name,
            &old_value,
            &new_value_vec,
        );

        true
    }

    pub fn global_inputs_set_image(
        &mut self,
        global_input_name: &str,
        new_value: &ImageValue,
    ) -> Result<(), GlobalInputsEngineError> {
        let Some(binding) = self.global_inputs_container.get_mut(global_input_name) else {
            return Err(GlobalInputsEngineError::BindingNotFound(
                global_input_name.to_string(),
            ));
        };

        let GlobalInputValue::Image { value } = &mut binding.r#type else {
            return Err(GlobalInputsEngineError::WrongGlobalInputType {
                global_input_name: global_input_name.to_string(),
                expected: "Image".to_string(),
            });
        };

        *value = new_value.clone();

        Ok(())
    }

    // ========================================================================
    // Get state machine input names for each type
    // ========================================================================

    pub fn get_state_machine_input_names_for_numeric(
        &self,
        binding_id: &str,
    ) -> Option<(Vec<String>, f32)> {
        let global_input = self.global_inputs_container.get(binding_id)?;

        let GlobalInputValue::Numeric { value } = &global_input.r#type else {
            return None;
        };

        let input_names = self.collect_state_machine_input_names(global_input)?;

        Some((input_names, *value))
    }

    pub fn get_state_machine_input_names_for_boolean(
        &self,
        binding_id: &str,
    ) -> Option<(Vec<String>, bool)> {
        let global_input = self.global_inputs_container.get(binding_id)?;

        let GlobalInputValue::Boolean { value } = &global_input.r#type else {
            return None;
        };

        let input_names = self.collect_state_machine_input_names(global_input)?;

        Some((input_names, *value))
    }

    pub fn get_state_machine_input_names_for_string(
        &self,
        binding_id: &str,
    ) -> Option<(Vec<String>, String)> {
        let global_input = self.global_inputs_container.get(binding_id)?;

        let GlobalInputValue::String { value } = &global_input.r#type else {
            return None;
        };

        let input_names = self.collect_state_machine_input_names(global_input)?;

        Some((input_names, value.clone()))
    }

    pub fn collect_all_state_machine_updates(
        &self,
    ) -> (
        Vec<(Vec<String>, bool)>,
        Vec<(Vec<String>, f32)>,
        Vec<(Vec<String>, String)>,
    ) {
        let mut boolean_updates = Vec::new();
        let mut numeric_updates = Vec::new();
        let mut string_updates = Vec::new();

        for (binding_id, _) in self.global_inputs_container.iter() {
            if let Some(update) = self.get_state_machine_input_names_for_boolean(binding_id) {
                boolean_updates.push(update);
            }
            if let Some(update) = self.get_state_machine_input_names_for_numeric(binding_id) {
                numeric_updates.push(update);
            }
            if let Some(update) = self.get_state_machine_input_names_for_string(binding_id) {
                string_updates.push(update);
            }
        }

        (boolean_updates, numeric_updates, string_updates)
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

    pub fn observe_string_global_input_value_change(
        &self,
        input_name: &str,
        old_value: &str,
        new_value: &str,
    ) {
        if old_value == new_value {
            return;
        }
        if let Ok(observers) = self.observers.try_read() {
            for observer in observers.iter() {
                observer.on_string_global_input_value_change(
                    input_name.to_string(),
                    old_value.to_string(),
                    new_value.to_string(),
                );
            }
        }
    }

    pub fn observe_color_global_input_value_change(
        &self,
        input_name: &str,
        old_value: &Vec<f32>,
        new_value: &Vec<f32>,
    ) {
        if old_value == new_value {
            return;
        }
        if let Ok(observers) = self.observers.try_read() {
            for observer in observers.iter() {
                observer.on_color_global_input_value_change(
                    input_name.to_string(),
                    old_value.clone(),
                    new_value.clone(),
                );
            }
        }
    }

    pub fn observe_gradient_global_input_value_change(
        &self,
        input_name: &str,
        old_value: &Vec<f32>,
        new_value: &Vec<f32>,
    ) {
        if old_value == new_value {
            return;
        }
        if let Ok(observers) = self.observers.try_read() {
            for observer in observers.iter() {
                observer.on_gradient_global_input_value_change(
                    input_name.to_string(),
                    old_value.clone(),
                    new_value.clone(),
                );
            }
        }
    }

    pub fn observe_numeric_global_input_value_change(
        &self,
        input_name: &str,
        old_value: f32,
        new_value: f32,
    ) {
        if old_value == new_value {
            return;
        }
        if let Ok(observers) = self.observers.try_read() {
            for observer in observers.iter() {
                observer.on_numeric_global_input_value_change(
                    input_name.to_string(),
                    old_value,
                    new_value,
                );
            }
        }
    }

    pub fn observe_boolean_global_input_value_change(
        &self,
        input_name: &str,
        old_value: bool,
        new_value: bool,
    ) {
        if old_value == new_value {
            return;
        }
        if let Ok(observers) = self.observers.try_read() {
            for observer in observers.iter() {
                observer.on_boolean_global_input_value_change(
                    input_name.to_string(),
                    old_value,
                    new_value,
                );
            }
        }
    }

    pub fn observe_vector_global_input_value_change(
        &self,
        input_name: &str,
        old_value: [f32; 2],
        new_value: [f32; 2],
    ) {
        if old_value == new_value {
            return;
        }
        if let Ok(observers) = self.observers.try_read() {
            for observer in observers.iter() {
                observer.on_vector_global_input_value_change(
                    input_name.to_string(),
                    old_value,
                    new_value,
                );
            }
        }
    }

    pub fn subscribe(&self, observer: Arc<dyn GlobalInputsObserver>) {
        let mut observers = self.observers.write().unwrap();
        observers.push(observer);
    }

    pub fn unsubscribe(&self, observer: &Arc<dyn GlobalInputsObserver>) {
        self.observers
            .write()
            .unwrap()
            .retain(|o| !Arc::ptr_eq(o, observer));
    }
}
