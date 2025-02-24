#[cfg(target_os = "emscripten")]
mod em {
    extern "C" {
        pub fn emscripten_run_script(script: *const i8);
    }
}

pub struct NativeOpenURL;

impl NativeOpenURL {
    pub fn open_url(url: &str, target: &str) -> Result<(), String> {
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
            let result = webbrowser::open(url);
            let _ = target.to_lowercase();

            if result.is_err() {
                return Err("Failed to open browser".to_string());
            }

            return Ok(());
        }
    }
}
