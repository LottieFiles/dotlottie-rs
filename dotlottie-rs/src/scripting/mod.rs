use std::{rc::Rc, sync::RwLock};

use crate::DotLottiePlayerContainer;

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

/// Represents a scripting engine capable of evaluating JavaScript code.
pub struct ScriptingEngine {
    /// Evaluates a JavaScript source string and returns a result as a string, or an error.
    // fn eval(&mut self, source: &str) -> Result<String, ScriptingError>;
    pub player: Option<Rc<RwLock<DotLottiePlayerContainer>>>,
    has_registered_functions: bool,
}

impl ScriptingEngine {
    pub fn new(player: Rc<RwLock<DotLottiePlayerContainer>>) -> ScriptingEngine {
        ScriptingEngine {
            player: Some(player),
            // Todo: Self register inside eval
            has_registered_functions: false,
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

    pub fn eval(&self, script: &str) -> bool {
        println!("Evaluating script: {}", script);
        let script_bytes = script.as_bytes();
        let script_ptr = script_bytes.as_ptr();
        let script_size = script_bytes.len();

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

    pub fn set_theme(&self, theme_id: &str) -> bool {
        println!(
            "Jerry script successfully called set_theme with: {}",
            theme_id
        );

        if let Some(player) = &self.player {
            match player.read() {
                Ok(player_guard) => {
                    return player_guard.set_theme(theme_id);
                }
                Err(poison_error) => {
                    println!("Lock was poisoned, attempting recovery");
                    let player_guard = poison_error.into_inner();
                    return player_guard.set_theme(theme_id);
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
            match player.read() {
                Ok(player_guard) => {
                    return player_guard.set_frame(frame_no);
                }
                Err(poison_error) => {
                    println!("Lock was poisoned in set_frame, attempting recovery");
                    let player_guard = poison_error.into_inner();
                    return player_guard.set_frame(frame_no);
                }
            }
        }

        false
    }
}

// Add a cleanup method or implement Drop
impl Drop for ScriptingEngine {
    fn drop(&mut self) {
        unsafe {
            jerry::jerry_cleanup();
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
