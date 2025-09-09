use std::{rc::Rc, sync::RwLock};

use crate::{Config, DotLottiePlayerContainer, Layout, Mode};

#[expect(non_upper_case_globals)]
#[allow(non_snake_case)]
#[expect(non_camel_case_types)]
#[expect(dead_code)]
mod jerry {
    include!(concat!(env!("OUT_DIR"), "/jerry_script_bindings.rs"));
}

static ENGINE_NATIVE_INFO: jerry::jerry_object_native_info_t = jerry::jerry_object_native_info_t {
    free_cb: None,
    number_of_references: 0,
    offset_of_references: 0,
};

/// Scripting engine abstraction for dotlottie-rs.
/// This module is intended to provide a unified interface for running JavaScript code
/// (e.g., for Lottie expressions or scripting) using an embeddable JS engine such as JerryScript.
///
///
/// Notes: Currently register_functions needs to be called from the object creator
/// Represents a scripting engine capable of evaluating JavaScript code.
pub struct ScriptingEngine {
    pub player: Option<Rc<RwLock<DotLottiePlayerContainer>>>,
    is_initialized: bool,
}

impl ScriptingEngine {
    pub fn new(player: Rc<RwLock<DotLottiePlayerContainer>>) -> ScriptingEngine {
        let mut engine = ScriptingEngine {
            player: Some(player),
            is_initialized: false,
        };

        // Initialize JerryScript context once during creation
        // engine.initialize();
        engine
    }

    pub fn initialize(&mut self) {
        if self.is_initialized {
            return;
        }

        unsafe {
            jerry::jerry_init(0);
            self.register_functions();
            self.is_initialized = true;
        }
    }

    // pub fn new(player: Rc<RwLock<DotLottiePlayerContainer>>) -> ScriptingEngine {
    //     let engine = ScriptingEngine {
    //         player: Some(player),
    //     };

    //     engine.register_functions();

    //     engine
    // }

    pub fn register_functions(&self) {
        unsafe {
            jerry::jerry_init(0);

            let global_object = jerry::jerry_current_realm();
            println!("Global object: {}", global_object);

            // Use the static info structure
            jerry::jerry_object_set_native_ptr(
                global_object,
                &ENGINE_NATIVE_INFO,
                self as *const Self as *mut std::os::raw::c_void,
            );

            // Register setTheme function
            let property_name =
                jerry::jerry_string_sz(b"setTheme\0".as_ptr() as *const std::os::raw::c_char);
            let property_value_func = jerry::jerry_function_external(Some(Self::jerry_set_theme));
            let set_result =
                jerry::jerry_object_set(global_object, property_name, property_value_func);

            if jerry::jerry_value_is_exception(set_result) {
                eprintln!("Failed to add the 'setTheme' property");
            } else {
                println!("Successfully added the 'setTheme' property");
            }

            // Register play function
            let play_name =
                jerry::jerry_string_sz(b"play\0".as_ptr() as *const std::os::raw::c_char);
            let play_func = jerry::jerry_function_external(Some(Self::jerry_play));
            let play_result = jerry::jerry_object_set(global_object, play_name, play_func);

            if jerry::jerry_value_is_exception(play_result) {
                eprintln!("Failed to add the 'play' property");
            } else {
                println!("Successfully added the 'play' property");
            }

            // Register pause function
            let pause_name =
                jerry::jerry_string_sz(b"pause\0".as_ptr() as *const std::os::raw::c_char);
            let pause_func = jerry::jerry_function_external(Some(Self::jerry_pause));
            let pause_result = jerry::jerry_object_set(global_object, pause_name, pause_func);

            if jerry::jerry_value_is_exception(pause_result) {
                eprintln!("Failed to add the 'pause' property");
            } else {
                println!("Successfully added the 'pause' property");
            }

            // Register stop function
            let stop_name =
                jerry::jerry_string_sz(b"stop\0".as_ptr() as *const std::os::raw::c_char);
            let stop_func = jerry::jerry_function_external(Some(Self::jerry_stop));
            let stop_result = jerry::jerry_object_set(global_object, stop_name, stop_func);

            if jerry::jerry_value_is_exception(stop_result) {
                eprintln!("Failed to add the 'stop' property");
            } else {
                println!("Successfully added the 'stop' property");
            }

            // Register setFrame function
            let set_frame_name =
                jerry::jerry_string_sz(b"setFrame\0".as_ptr() as *const std::os::raw::c_char);
            let set_frame_func = jerry::jerry_function_external(Some(Self::jerry_set_frame));
            let set_frame_result =
                jerry::jerry_object_set(global_object, set_frame_name, set_frame_func);

            if jerry::jerry_value_is_exception(set_frame_result) {
                eprintln!("Failed to add the 'setFrame' property");
            } else {
                println!("Successfully added the 'setFrame' property");
            }

            // Register setConfig function
            let set_config_name =
                jerry::jerry_string_sz(b"setConfig\0".as_ptr() as *const std::os::raw::c_char);
            let set_config_func = jerry::jerry_function_external(Some(Self::jerry_set_config));
            let set_config_result =
                jerry::jerry_object_set(global_object, set_config_name, set_config_func);

            if jerry::jerry_value_is_exception(set_config_result) {
                eprintln!("Failed to add the 'setConfig' property");
            } else {
                println!("Successfully added the 'setConfig' property");
            }

            // Cleanup
            jerry::jerry_value_free(set_result);
            jerry::jerry_value_free(property_value_func);
            jerry::jerry_value_free(property_name);
            jerry::jerry_value_free(play_result);
            jerry::jerry_value_free(play_func);
            jerry::jerry_value_free(play_name);
            jerry::jerry_value_free(pause_result);
            jerry::jerry_value_free(pause_func);
            jerry::jerry_value_free(pause_name);
            jerry::jerry_value_free(stop_result);
            jerry::jerry_value_free(stop_func);
            jerry::jerry_value_free(stop_name);
            jerry::jerry_value_free(set_frame_result);
            jerry::jerry_value_free(set_frame_func);
            jerry::jerry_value_free(set_frame_name);
            jerry::jerry_value_free(set_config_result);
            jerry::jerry_value_free(set_config_func);
            jerry::jerry_value_free(set_config_name);
            jerry::jerry_value_free(global_object);
        }
    }

    // Jerry callback functions
    unsafe extern "C" fn jerry_play(
        _call_info_p: *const jerry::jerry_call_info_t,
        _arguments: *const jerry::jerry_value_t,
        _argument_count: jerry::jerry_length_t,
    ) -> jerry::jerry_value_t {
        println!("jerry_play called");

        unsafe {
            let global_object = jerry::jerry_current_realm();
            let engine_ptr = jerry::jerry_object_get_native_ptr(global_object, &ENGINE_NATIVE_INFO);
            jerry::jerry_value_free(global_object);

            if !engine_ptr.is_null() {
                let engine = &*(engine_ptr as *const ScriptingEngine);
                let success = engine.play();
                return jerry::jerry_boolean(success);
            } else {
                println!("Engine pointer is null in jerry_play!");
            }
        }

        jerry::jerry_boolean(false)
    }

    unsafe extern "C" fn jerry_set_frame(
        _call_info_p: *const jerry::jerry_call_info_t,
        arguments: *const jerry::jerry_value_t,
        argument_count: jerry::jerry_length_t,
    ) -> jerry::jerry_value_t {
        println!("jerry_set_frame called with {} arguments", argument_count);

        if argument_count > 0 {
            unsafe {
                let global_object = jerry::jerry_current_realm();
                let engine_ptr =
                    jerry::jerry_object_get_native_ptr(global_object, &ENGINE_NATIVE_INFO);
                jerry::jerry_value_free(global_object);

                if !engine_ptr.is_null() {
                    let engine = &*(engine_ptr as *const ScriptingEngine);

                    // Convert the first argument to a number (f32)
                    let frame_number = jerry::jerry_value_as_number(*arguments);

                    println!("Calling set_frame with: {}", frame_number);
                    let success = engine.set_frame(frame_number);
                    return jerry::jerry_boolean(success);
                } else {
                    println!("Engine pointer is null in jerry_set_frame!");
                }
            }
        } else {
            println!("jerry_set_frame called with no arguments!");
        }

        jerry::jerry_boolean(false)
    }

    unsafe extern "C" fn jerry_pause(
        _call_info_p: *const jerry::jerry_call_info_t,
        _arguments: *const jerry::jerry_value_t,
        _argument_count: jerry::jerry_length_t,
    ) -> jerry::jerry_value_t {
        println!("jerry_pause called");

        unsafe {
            let global_object = jerry::jerry_current_realm();
            let engine_ptr = jerry::jerry_object_get_native_ptr(global_object, &ENGINE_NATIVE_INFO);
            jerry::jerry_value_free(global_object);

            if !engine_ptr.is_null() {
                let engine = &*(engine_ptr as *const ScriptingEngine);
                let success = engine.pause();
                return jerry::jerry_boolean(success);
            } else {
                println!("Engine pointer is null in jerry_pause!");
            }
        }

        jerry::jerry_boolean(false)
    }

    unsafe extern "C" fn jerry_stop(
        _call_info_p: *const jerry::jerry_call_info_t,
        _arguments: *const jerry::jerry_value_t,
        _argument_count: jerry::jerry_length_t,
    ) -> jerry::jerry_value_t {
        println!("jerry_stop called");

        unsafe {
            let global_object = jerry::jerry_current_realm();
            let engine_ptr = jerry::jerry_object_get_native_ptr(global_object, &ENGINE_NATIVE_INFO);
            jerry::jerry_value_free(global_object);

            if !engine_ptr.is_null() {
                let engine = &*(engine_ptr as *const ScriptingEngine);
                let success = engine.stop();
                return jerry::jerry_boolean(success);
            } else {
                println!("Engine pointer is null in jerry_stop!");
            }
        }

        jerry::jerry_boolean(false)
    }

    unsafe extern "C" fn jerry_set_theme(
        _call_info_p: *const jerry::jerry_call_info_t,
        arguments: *const jerry::jerry_value_t,
        argument_count: jerry::jerry_length_t,
    ) -> jerry::jerry_value_t {
        println!("jerry_set_theme called with {} arguments", argument_count);

        if argument_count > 0 {
            unsafe {
                let global_object = jerry::jerry_current_realm();

                // Use the same static reference
                let engine_ptr =
                    jerry::jerry_object_get_native_ptr(global_object, &ENGINE_NATIVE_INFO);
                jerry::jerry_value_free(global_object);

                if !engine_ptr.is_null() {
                    let engine = &*(engine_ptr as *const ScriptingEngine);

                    // Convert the first argument to string
                    let arg_as_string = jerry::jerry_value_to_string(*arguments);
                    let string_length = jerry::jerry_string_length(arg_as_string);

                    if string_length > 0 {
                        let mut buffer = vec![0u8; string_length as usize + 1];
                        jerry::jerry_string_to_buffer(
                            arg_as_string,
                            jerry::jerry_encoding_t_JERRY_ENCODING_UTF8,
                            buffer.as_mut_ptr(),
                            string_length,
                        );

                        if let Ok(theme_id) =
                            String::from_utf8(buffer[..string_length as usize].to_vec())
                        {
                            println!("Calling set_theme with: {}", theme_id);
                            let success = engine.set_theme(&theme_id);
                            jerry::jerry_value_free(arg_as_string);

                            return jerry::jerry_boolean(success);
                        }
                    }
                    jerry::jerry_value_free(arg_as_string);
                } else {
                    println!("Engine pointer is null!");
                }
            }
        }

        jerry::jerry_undefined()
    }

    unsafe extern "C" fn jerry_set_config(
        _call_info_p: *const jerry::jerry_call_info_t,
        arguments: *const jerry::jerry_value_t,
        argument_count: jerry::jerry_length_t,
    ) -> jerry::jerry_value_t {
        println!("jerry_set_config called with {} arguments", argument_count);

        if argument_count > 0 {
            unsafe {
                let global_object = jerry::jerry_current_realm();
                let engine_ptr =
                    jerry::jerry_object_get_native_ptr(global_object, &ENGINE_NATIVE_INFO);
                jerry::jerry_value_free(global_object);

                if !engine_ptr.is_null() {
                    let engine = &*(engine_ptr as *const ScriptingEngine);

                    // Parse the JavaScript object into a Config struct
                    if let Some(config) = Self::parse_config_from_jerry_object(*arguments) {
                        println!("Calling set_config with parsed config");
                        let success = engine.set_config(config);
                        return jerry::jerry_boolean(success);
                    } else {
                        println!("Failed to parse config object!");
                    }
                } else {
                    println!("Engine pointer is null in jerry_set_config!");
                }
            }
        } else {
            println!("jerry_set_config called with no arguments!");
        }

        jerry::jerry_boolean(false)
    }

    // Add this helper function to parse the JavaScript object:

    unsafe fn parse_config_from_jerry_object(js_object: jerry::jerry_value_t) -> Option<Config> {
        if !jerry::jerry_value_is_object(js_object) {
            println!("Argument is not an object!");
            return None;
        }

        // Helper function to get a property as a string
        let get_string_property = |obj: jerry::jerry_value_t, prop_name: &str| -> Option<String> {
            let prop_name_jerry = jerry::jerry_string_sz(
                format!("{}\0", prop_name).as_ptr() as *const std::os::raw::c_char
            );
            let prop_value = jerry::jerry_object_get(obj, prop_name_jerry);

            let result = if jerry::jerry_value_is_string(prop_value) {
                let string_length = jerry::jerry_string_length(prop_value);
                let mut buffer = vec![0u8; string_length as usize + 1];
                jerry::jerry_string_to_buffer(
                    prop_value,
                    jerry::jerry_encoding_t_JERRY_ENCODING_UTF8,
                    buffer.as_mut_ptr(),
                    string_length,
                );
                String::from_utf8(buffer[..string_length as usize].to_vec()).ok()
            } else {
                None
            };

            jerry::jerry_value_free(prop_value);
            jerry::jerry_value_free(prop_name_jerry);
            result
        };

        // Helper function to get a property as a number
        let get_number_property = |obj: jerry::jerry_value_t, prop_name: &str| -> Option<f32> {
            let prop_name_jerry = jerry::jerry_string_sz(
                format!("{}\0", prop_name).as_ptr() as *const std::os::raw::c_char
            );
            let prop_value = jerry::jerry_object_get(obj, prop_name_jerry);

            let result = if jerry::jerry_value_is_number(prop_value) {
                Some(jerry::jerry_value_as_number(prop_value))
            } else {
                None
            };

            jerry::jerry_value_free(prop_value);
            jerry::jerry_value_free(prop_name_jerry);
            result
        };

        // Helper function to get a property as a boolean
        let get_boolean_property = |obj: jerry::jerry_value_t, prop_name: &str| -> Option<bool> {
            let prop_name_jerry = jerry::jerry_string_sz(
                format!("{}\0", prop_name).as_ptr() as *const std::os::raw::c_char
            );
            let prop_value = jerry::jerry_object_get(obj, prop_name_jerry);

            let result = if jerry::jerry_value_is_boolean(prop_value) {
                Some(jerry::jerry_value_is_true(prop_value))
            } else {
                None
            };

            jerry::jerry_value_free(prop_value);
            jerry::jerry_value_free(prop_name_jerry);
            result
        };

        // Helper function to get an array property as Vec<f32>
        let get_array_property = |obj: jerry::jerry_value_t, prop_name: &str| -> Option<Vec<f32>> {
            let prop_name_jerry = jerry::jerry_string_sz(
                format!("{}\0", prop_name).as_ptr() as *const std::os::raw::c_char
            );
            let prop_value = jerry::jerry_object_get(obj, prop_name_jerry);

            let result = if jerry::jerry_value_is_array(prop_value) {
                let length_prop =
                    jerry::jerry_string_sz(b"length\0".as_ptr() as *const std::os::raw::c_char);
                let length_value = jerry::jerry_object_get(prop_value, length_prop);

                if jerry::jerry_value_is_number(length_value) {
                    let length = jerry::jerry_value_as_number(length_value) as u32;
                    let mut vec = Vec::new();

                    for i in 0..length {
                        let index_value = jerry::jerry_object_get_index(prop_value, i);
                        if jerry::jerry_value_is_number(index_value) {
                            vec.push(jerry::jerry_value_as_number(index_value));
                        }
                        jerry::jerry_value_free(index_value);
                    }

                    jerry::jerry_value_free(length_value);
                    jerry::jerry_value_free(length_prop);
                    Some(vec)
                } else {
                    jerry::jerry_value_free(length_value);
                    jerry::jerry_value_free(length_prop);
                    None
                }
            } else {
                None
            };

            jerry::jerry_value_free(prop_value);
            jerry::jerry_value_free(prop_name_jerry);
            result
        };

        // Parse each field with defaults where appropriate
        // Note: You'll need to define default values based on your Config struct's needs
        let mode = Mode::Forward; // You'll need to parse this based on your Mode enum
        let loop_animation = get_boolean_property(js_object, "loop_animation").unwrap_or(false);
        let speed = get_number_property(js_object, "speed").unwrap_or(1.0);
        let use_frame_interpolation =
            get_boolean_property(js_object, "use_frame_interpolation").unwrap_or(true);
        let autoplay = get_boolean_property(js_object, "autoplay").unwrap_or(false);
        let segment = get_array_property(js_object, "segment").unwrap_or_else(Vec::new);
        // let background_color =
        //     get_number_property(js_object, "background_color").unwrap_or(0.0) as u32;
        let background_color = 0.0 as u32;
        let layout = Layout::default(); // You'll need to parse this based on your Layout enum
        let marker = get_string_property(js_object, "marker").unwrap_or_else(|| String::new());
        let theme_id = get_string_property(js_object, "theme_id").unwrap_or_else(|| String::new());
        let animation_id =
            get_string_property(js_object, "animation_id").unwrap_or_else(|| String::new());
        let state_machine_id =
            get_string_property(js_object, "state_machine_id").unwrap_or_else(|| String::new());

        println!(">>> Animation id: {}", animation_id);
        Some(Config {
            mode,
            loop_animation,
            speed,
            use_frame_interpolation,
            autoplay,
            segment,
            background_color,
            layout,
            marker,
            theme_id,
            animation_id,
            state_machine_id,
        })
    }

    pub fn eval(&self, script: &str, force_global: bool) -> bool {
        // if !self.is_initialized {
        //     eprintln!("ScriptingEngine not initialized!");
        //     return false;
        // }

        // println!("Evaluating script: {}", script);
        // let script_bytes = script.as_bytes();
        // let script_ptr = script_bytes.as_ptr();
        // let script_size = script_bytes.len();

        let (script_bytes, wrapped_script);
        let (script_ptr, script_size) = if force_global {
            // Force execution in global context
            wrapped_script = format!("(function(){{ {} }}).call(this);", script);
            script_bytes = wrapped_script.as_bytes();
            (script_bytes.as_ptr(), script_bytes.len())
        } else {
            // Execute in local context
            let script_bytes = script.as_bytes();
            (script_bytes.as_ptr(), script_bytes.len())
        };

        unsafe {
            let eval_ret = jerry::jerry_eval(
                script_ptr,
                script_size,
                jerry::jerry_parse_option_enable_feature_t_JERRY_PARSE_NO_OPTS,
            );

            let run_ok = !jerry::jerry_value_is_exception(eval_ret);

            if !run_ok {
                // Get error details
                let error_str = jerry::jerry_value_to_string(eval_ret);
                let error_length = jerry::jerry_string_length(error_str);
                let mut error_buffer = vec![0u8; error_length as usize + 1];
                jerry::jerry_string_to_buffer(
                    error_str,
                    jerry::jerry_encoding_t_JERRY_ENCODING_UTF8,
                    error_buffer.as_mut_ptr(),
                    error_length,
                );
                if let Ok(error_msg) =
                    String::from_utf8(error_buffer[..error_length as usize].to_vec())
                {
                    println!("JavaScript error: {}", error_msg);
                }
                jerry::jerry_value_free(error_str);
            }

            jerry::jerry_value_free(eval_ret);

            if run_ok {
                println!("Script executed successfully!");
                true
            } else {
                eprintln!("Script execution failed!");
                false
            }
        }
    }

    pub fn eval_and_get_variable(&self, var_name: &str) -> Option<f32> {
        if !self.is_initialized {
            return None;
        }

        unsafe {
            let global_object = jerry::jerry_current_realm();
            let prop_name = jerry::jerry_string_sz(
                format!("{}\0", var_name).as_ptr() as *const std::os::raw::c_char
            );

            let prop_value = jerry::jerry_object_get(global_object, prop_name);
            let result = if jerry::jerry_value_is_number(prop_value) {
                Some(jerry::jerry_value_as_number(prop_value))
            } else {
                None
            };

            jerry::jerry_value_free(prop_value);
            jerry::jerry_value_free(prop_name);
            jerry::jerry_value_free(global_object);

            result
        }
    }

    pub fn reset_context(&mut self) {
        if self.is_initialized {
            unsafe {
                jerry::jerry_cleanup();
            }
            self.is_initialized = false;
        }
        self.initialize();
    }

    pub fn set_theme(&self, theme_id: &str) -> bool {
        println!(
            ">> Jerry script successfully called set_theme with: {}",
            theme_id
        );

        if let Some(player) = &self.player {
            // Use try_read instead of read to avoid blocking
            match player.try_read() {
                Ok(player_guard) => {
                    return player_guard.set_theme(theme_id);
                }
                Err(_) => {
                    println!("Could not acquire read lock in set_frame - possible deadlock!");
                    return false;
                }
            }
        }

        false
    }

    // Rust methods that interact with the player
    pub fn play(&self) -> bool {
        println!("Jerry script successfully called play");

        if let Some(player) = &self.player {
            match player.read() {
                Ok(player_guard) => {
                    return player_guard.play();
                }
                Err(poison_error) => {
                    println!("Lock was poisoned in play, attempting recovery");
                    let player_guard = poison_error.into_inner();
                    return player_guard.play();
                }
            }
        }

        false
    }

    pub fn pause(&self) -> bool {
        println!("Jerry script successfully called pause");

        if let Some(player) = &self.player {
            match player.read() {
                Ok(player_guard) => {
                    return player_guard.pause();
                }
                Err(poison_error) => {
                    println!("Lock was poisoned in pause, attempting recovery");
                    let player_guard = poison_error.into_inner();
                    return player_guard.pause();
                }
            }
        }

        false
    }

    pub fn stop(&self) -> bool {
        println!("Jerry script successfully called stop");

        if let Some(player) = &self.player {
            match player.read() {
                Ok(player_guard) => {
                    return player_guard.stop();
                }
                Err(poison_error) => {
                    println!("Lock was poisoned in stop, attempting recovery");
                    let player_guard = poison_error.into_inner();
                    return player_guard.stop();
                }
            }
        }

        false
    }

    pub fn set_frame(&self, frame_no: f32) -> bool {
        println!(
            "Jerry script successfully called set_frame with: {}",
            frame_no
        );

        if let Some(player) = &self.player {
            // Use try_read instead of read to avoid blocking
            match player.try_read() {
                Ok(player_guard) => {
                    return player_guard.set_frame(frame_no);
                }
                Err(_) => {
                    println!("Could not acquire read lock in set_frame - possible deadlock!");
                    return false;
                }
            }
        }

        false
    }

    pub fn set_config(&self, config: Config) -> bool {
        println!("Jerry script successfully called set_config");
        println!("-----");
        println!("{:?}", config);

        if let Some(player) = &self.player {
            match player.try_write() {
                Ok(player_guard) => {
                    player_guard.set_config(config);
                    return true;
                }
                Err(_) => {
                    println!("Could not acquire write lock in set_config - possible deadlock!");
                    return false;
                }
            }
        }

        false
    }
}

// Add a cleanup method or implement Drop
impl Drop for ScriptingEngine {
    fn drop(&mut self) {
        if self.is_initialized {
            // Add this check!
            unsafe {
                jerry::jerry_cleanup();
            }
        }
    }
}
/// Represents errors that can occur during scripting operations.
#[derive(Debug)]
pub enum ScriptingError {
    EngineInitFailed,
    EvalFailed(String),
    InternalError(String),
}

impl std::fmt::Display for ScriptingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptingError::EngineInitFailed => write!(f, "Failed to initialize scripting engine"),
            ScriptingError::EvalFailed(msg) => write!(f, "Script evaluation failed: {msg}"),
            ScriptingError::InternalError(msg) => write!(f, "Internal scripting error: {msg}"),
        }
    }
}

impl std::error::Error for ScriptingError {}
