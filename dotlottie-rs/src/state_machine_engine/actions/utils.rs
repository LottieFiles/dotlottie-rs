use crate::StateMachineEngine;

#[cfg(target_os = "emscripten")]
mod em {
    extern "C" {
        pub fn emscripten_run_script(script: *const i8);
    }
}

pub struct NativeOpenUrl;

impl NativeOpenUrl {
    pub fn open_url(url: &str, target: &str, engine: &StateMachineEngine) -> Result<(), String> {
        #[cfg(target_os = "emscripten")]
        unsafe {
            use std::ffi::CString;

            let command = if target.is_empty() {
                format!("window.open('{}');", url)
            } else {
                format!("window.open('{}', '{}');", url, target)
            };

            let command_cstr = CString::new(command).unwrap();
            em::emscripten_run_script(command_cstr.as_ptr() as *const i8);

            return Ok(());
        }

        #[cfg(not(target_os = "emscripten"))]
        {
            let _ = target.to_lowercase();
            let command = format!("OpenUrl: {}", url);
            engine.observe_framework_open_url_event(&command);
            return Ok(());
        }
    }
}
