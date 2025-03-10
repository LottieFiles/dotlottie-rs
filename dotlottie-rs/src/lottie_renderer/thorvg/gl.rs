use super::common::{BackendInstances, TvgEngineInit};
use super::types::*;
use super::*;

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[allow(clippy::all)]
#[cfg(target_arch = "wasm32")]
mod em {
    use std::ffi::CString;
    use std::os::raw::{c_int, c_void};

    #[allow(non_snake_case)]
    #[allow(non_upper_case_globals)]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    #[allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/emscripten_bindings.rs"));

    pub type EmscriptenResult = ::std::os::raw::c_int;
    pub type EmscriptenWebglContextHandle = usize;
    pub type EmscriptenWebglContextProxyMode = ::std::os::raw::c_int;
    pub type EmWebglPowerPreference = ::std::os::raw::c_int;
    pub const EMSCRIPTEN_WEBGL_CONTEXT_PROXY_MODE_OFF: EmscriptenWebglContextProxyMode = 0;
    pub const EMSCRIPTEN_WEBGL_CONTEXT_PROXY_MODE_ALWAYS: EmscriptenWebglContextProxyMode = 2;
    pub const EM_WEBGL_CONTEXT_VERSION_MAJOR: i32 = 2;
    pub const EM_WEBGL_CONTEXT_VERSION_MINOR: i32 = 0;

    extern "C" {
        pub fn emscripten_webgl_create_context(
            canvas_selector: *const ::std::os::raw::c_char,
            attrs: *const EmscriptenWebGLContextAttributes,
        ) -> EmscriptenWebglContextHandle;
        pub fn emscripten_webgl_destroy_context(context: EmscriptenWebglContextHandle);
        pub fn emscripten_webgl_make_context_current(
            context: EmscriptenWebglContextHandle,
        ) -> EmscriptenResult;
    }

    #[derive(Clone)]
    pub struct WebGLContext {
        pub ctx: Option<EmscriptenWebglContextHandle>,
    }

    impl WebGLContext {
        pub fn new(canvas_selector: &str) -> Self {
            let webgl_context_attributes = EmscriptenWebGLContextAttributes {
                alpha: true,
                depth: false,
                stencil: false,
                antialias: false,
                premultipliedAlpha: true,
                preserveDrawingBuffer: false,
                powerPreference: EM_WEBGL_POWER_PREFERENCE_DEFAULT as i32,
                failIfMajorPerformanceCaveat: false,
                majorVersion: EM_WEBGL_CONTEXT_VERSION_MAJOR,
                minorVersion: EM_WEBGL_CONTEXT_VERSION_MINOR,
                enableExtensionsByDefault: true,
                explicitSwapControl: false,
                proxyContextToMainThread: EMSCRIPTEN_WEBGL_CONTEXT_PROXY_MODE_OFF,
                renderViaOffscreenBackBuffer: false,
            };

            let canvas_selector_cstr = CString::new(canvas_selector).unwrap();

            let webgl_context_handle = unsafe {
                emscripten_webgl_create_context(
                    canvas_selector_cstr.as_ptr(),
                    &webgl_context_attributes,
                )
            };

            if webgl_context_handle == 0 {
                return Self { ctx: None };
            }

            let ctx = Self {
                ctx: Some(webgl_context_handle),
            };

            ctx.make_current();

            ctx
        }

        pub fn make_current(&self) -> EmscriptenWebglContextHandle {
            unsafe { emscripten_webgl_make_context_current(self.ctx.unwrap()) };

            self.ctx.unwrap().clone()
        }
    }

    impl Drop for WebGLContext {
        fn drop(&mut self) {
            if let Some(ctx) = self.ctx {
                unsafe { emscripten_webgl_destroy_context(ctx) };
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
static INSTANCES: BackendInstances = BackendInstances::new();

#[cfg(target_arch = "wasm32")]
pub struct GlBackend {
    context: em::WebGLContext,
}

#[cfg(target_arch = "wasm32")]
impl TvgEngineInit for GlBackend {
    const ENGINE: TvgEngine = TvgEngine::TvgEngineGl;
}

#[cfg(target_arch = "wasm32")]
impl GlBackend {
    pub fn new(threads: u32, html_canvas_selector: &str) -> Self {
        let context = em::WebGLContext::new(html_canvas_selector);

        INSTANCES.init::<Self>(threads).unwrap();

        Self { context }
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for GlBackend {
    fn drop(&mut self) {
        INSTANCES.terminate::<Self>();
    }
}

#[cfg(target_arch = "wasm32")]
impl TvgBackend for GlBackend {
    fn create_canvas(&self) -> *mut tvg::Tvg_Canvas {
        unsafe { tvg::tvg_glcanvas_create() }
    }

    fn set_target(
        &self,
        canvas: *mut tvg::Tvg_Canvas,
        _buffer: &mut Vec<u32>,
        _stride: u32,
        width: u32,
        height: u32,
        color_space: ColorSpace,
    ) -> Result<(), TvgError> {
        let ctx = self.context.ctx.ok_or(TvgError::InvalidArgument)?;
        unsafe {
            tvg::tvg_glcanvas_set_target(
                canvas,
                ctx as *mut ::std::os::raw::c_void,
                0,
                width,
                height,
                color_space.into(),
            )
            .into_result()
        }
    }
}
