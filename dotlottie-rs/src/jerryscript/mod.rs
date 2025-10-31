#[expect(non_upper_case_globals)]
#[allow(non_snake_case)]
#[expect(non_camel_case_types)]
#[expect(dead_code)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/jerryscript_bindings.rs"));
}

use ffi::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// Errors that can occur when working with JerryScript
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Error {
    /// JavaScript exception occurred
    Exception(String),
    /// UTF-8 conversion error
    Utf8,
    /// Engine not initialized
    NotInitialized,
    /// Engine already initialized
    AlreadyInitialized,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Exception(msg) => write!(f, "JavaScript exception: {msg}"),
            Error::Utf8 => write!(f, "UTF-8 conversion error"),
            Error::NotInitialized => write!(f, "JerryScript engine not initialized"),
            Error::AlreadyInitialized => write!(f, "JerryScript engine already initialized"),
        }
    }
}

impl std::error::Error for Error {}

/// RAII wrapper for JerryScript values with automatic memory management
pub struct Value {
    handle: jerry_value_t,
}

#[allow(dead_code)]
impl Value {
    /// Create a new Value from a raw jerry_value_t handle
    fn new(handle: jerry_value_t) -> Self {
        Self { handle }
    }

    /// Create an undefined value
    pub fn undefined() -> Self {
        unsafe { Self::new(jerry_undefined()) }
    }

    /// Create a boolean value
    pub fn boolean(value: bool) -> Self {
        unsafe { Self::new(jerry_boolean(value)) }
    }

    /// Create a number value
    pub fn number(value: f32) -> Self {
        unsafe { Self::new(jerry_number(value)) }
    }

    /// Create a string value from a C string
    pub fn string(value: &str) -> Result<Self, Error> {
        use std::ffi::CString;
        let c_str = CString::new(value).map_err(|_| Error::Utf8)?;
        unsafe { Ok(Self::new(jerry_string_sz(c_str.as_ptr()))) }
    }

    /// Check if the value is undefined
    pub fn is_undefined(&self) -> bool {
        unsafe { jerry_value_is_undefined(self.handle) }
    }

    /// Check if the value is a number
    pub fn is_number(&self) -> bool {
        unsafe { jerry_value_is_number(self.handle) }
    }

    /// Check if the value is a string
    pub fn is_string(&self) -> bool {
        unsafe { jerry_value_is_string(self.handle) }
    }

    /// Check if the value is an object
    pub fn is_object(&self) -> bool {
        unsafe { jerry_value_is_object(self.handle) }
    }

    /// Check if the value is an exception
    pub fn is_exception(&self) -> bool {
        unsafe { jerry_value_is_exception(self.handle) }
    }

    /// Check if the value is a boolean (by process of elimination)
    pub fn is_boolean(&self) -> bool {
        !self.is_undefined()
            && !self.is_number()
            && !self.is_string()
            && !self.is_object()
            && !self.is_exception()
    }

    /// Convert value to a number
    pub fn to_number(&self) -> f32 {
        unsafe {
            // Try the direct conversion first
            let result = jerry_value_as_number(self.handle);

            // If it's a boolean type, jerry_value_as_number might return 0 for both true and false
            // Let's check if this is actually a boolean by process of elimination
            if result == 0.0 && self.is_boolean() {
                // For booleans, we need to determine true vs false differently
                // Create true and false values to compare against
                let true_val = jerry_boolean(true);
                let false_val = jerry_boolean(false);

                let is_true = self.handle == true_val;

                jerry_value_free(true_val);
                jerry_value_free(false_val);

                if is_true {
                    1.0
                } else {
                    0.0
                }
            } else {
                result
            }
        }
    }

    /// Convert value to a string representation
    ///
    /// This function converts the JavaScript value to its string representation,
    /// similar to calling `toString()` in JavaScript.
    pub fn to_string(&self) -> Result<String, Error> {
        unsafe {
            let string_val = jerry_value_to_string(self.handle);
            let result = Self::extract_string(string_val);
            jerry_value_free(string_val);
            result
        }
    }

    /// Extract a string from a jerry_value_t string
    fn extract_string(string_val: jerry_value_t) -> Result<String, Error> {
        unsafe {
            let length = jerry_string_length(string_val);
            let mut buffer = vec![0u8; (length + 1) as usize];
            let copied = jerry_string_to_buffer(
                string_val,
                jerry_encoding_t_JERRY_ENCODING_UTF8,
                buffer.as_mut_ptr(),
                length,
            );

            if copied == 0 {
                return Err(Error::Utf8);
            }

            buffer.truncate(copied as usize);
            String::from_utf8(buffer).map_err(|_| Error::Utf8)
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe {
            jerry_value_free(self.handle);
        }
    }
}

// Thread safety: JerryScript contexts can be moved between threads
unsafe impl Send for Value {}
unsafe impl Sync for Value {}

/// JerryScript engine context with RAII initialization and cleanup
pub struct Context {
    _private: (), // Prevent direct construction
}

// Global state to track initialization
static ENGINE_INITIALIZED: AtomicBool = AtomicBool::new(false);
static ENGINE_MUTEX: Mutex<()> = Mutex::new(());

#[allow(dead_code)]
impl Context {
    /// Initialize JerryScript engine with default flags
    pub fn new() -> Result<Self, Error> {
        Self::new_with_flags(jerry_init_flag_t_JERRY_INIT_EMPTY)
    }

    /// Initialize JerryScript engine with custom flags
    pub fn new_with_flags(flags: jerry_init_flag_t) -> Result<Self, Error> {
        let _guard = ENGINE_MUTEX.lock().map_err(|_| Error::NotInitialized)?;

        if ENGINE_INITIALIZED.load(Ordering::Acquire) {
            return Err(Error::AlreadyInitialized);
        }

        unsafe {
            jerry_init(flags);
        }

        ENGINE_INITIALIZED.store(true, Ordering::Release);

        Ok(Self { _private: () })
    }

    pub fn set_global_boolean(&self, name: &str, value: bool) -> Result<(), Error> {
        let js_code = format!("var {} = {};", name, if value { "true" } else { "false" });
        self.eval(&js_code)?;
        Ok(())
    }

    pub fn set_global_number(&self, name: &str, value: f32) -> Result<(), Error> {
        let js_code = format!("var {name} = {value};");
        self.eval(&js_code)?;
        Ok(())
    }

    pub fn set_global_string(&self, name: &str, value: &str) -> Result<(), Error> {
        // Escape the string value to handle quotes and special characters
        let escaped_value = value.replace('\\', "\\\\").replace('"', "\\\"");
        let js_code = format!("var {name} = \"{escaped_value}\";");
        self.eval(&js_code)?;
        Ok(())
    }

    pub fn get_global(&self, name: &str) -> Result<Value, Error> {
        self.eval(name)
    }

    pub fn eval(&self, source: &str) -> Result<Value, Error> {
        unsafe {
            let result = jerry_eval(
                source.as_ptr(),
                source.len(),
                0, // No special flags
            );

            if jerry_value_is_exception(result) {
                let error_msg = Value::new(result)
                    .to_string()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                jerry_value_free(result);
                Err(Error::Exception(error_msg))
            } else {
                Ok(Value::new(result))
            }
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        let _guard = ENGINE_MUTEX.lock();

        if ENGINE_INITIALIZED.load(Ordering::Acquire) {
            unsafe {
                jerry_cleanup();
            }
            ENGINE_INITIALIZED.store(false, Ordering::Release);
        }
    }
}

// Thread safety: Contexts can be moved between threads
unsafe impl Send for Context {}
unsafe impl Sync for Context {}
