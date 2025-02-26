use super::common::{BackendInstances, TvgEngineInit};
use super::*;
use std::os::raw::c_void;

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[allow(clippy::all)]
#[cfg(target_arch = "wasm32")]
mod em {
    use std::ffi::CString;
    use std::os::raw::c_void;
    use std::ptr;

    #[allow(non_snake_case)]
    #[allow(non_upper_case_globals)]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    #[allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/emscripten_bindings.rs"));
    extern "C" {
        pub fn wgpuCreateInstance(desc: *const c_void) -> WGPUInstance;
        pub fn wgpuInstanceCreateSurface(
            instance: WGPUInstance,
            desc: *const WGPUSurfaceDescriptor,
        ) -> WGPUSurface;
        pub fn wgpuSurfaceRelease(surface: WGPUSurface);
        pub fn emscripten_webgpu_get_device() -> WGPUDevice;
    }

    pub static mut WEBGPU_INSTANCE: WGPUInstance = ptr::null_mut();
    pub static mut WEBGPU_DEVICE: WGPUDevice = ptr::null_mut();

    pub struct WebGPUContext {
        pub surface: WGPUSurface,
    }

    impl WebGPUContext {
        pub fn new(selector: &str) -> Self {
            unsafe {
                WEBGPU_INSTANCE = wgpuCreateInstance(ptr::null());
                WEBGPU_DEVICE = emscripten_webgpu_get_device();
            }

            let mut canvas_desc: WGPUSurfaceDescriptorFromCanvasHTMLSelector =
                unsafe { std::mem::zeroed() };

            canvas_desc.chain.next = ptr::null_mut();
            canvas_desc.chain.sType = WGPUSType_WGPUSType_SurfaceDescriptorFromCanvasHTMLSelector;

            let selector_cstr = CString::new(selector).unwrap();
            canvas_desc.selector = selector_cstr.as_ptr();

            let mut surface_desc: WGPUSurfaceDescriptor = unsafe { std::mem::zeroed() };
            surface_desc.nextInChain = &canvas_desc.chain as *const _ as *const WGPUChainedStruct;

            let surface = unsafe { wgpuInstanceCreateSurface(WEBGPU_INSTANCE, &surface_desc) };

            Self { surface }
        }

        pub fn device(&self) -> WGPUDevice {
            unsafe { WEBGPU_DEVICE }
        }

        pub fn instance(&self) -> WGPUInstance {
            unsafe { WEBGPU_INSTANCE }
        }

        pub fn surface(&self) -> WGPUSurface {
            self.surface
        }
    }

    impl Drop for WebGPUContext {
        fn drop(&mut self) {
            unsafe {
                wgpuSurfaceRelease(self.surface);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
static INSTANCES: BackendInstances = BackendInstances::new();

#[cfg(target_arch = "wasm32")]
pub struct WgBackend {
    context: em::WebGPUContext,
}

#[cfg(target_arch = "wasm32")]
impl TvgEngineInit for WgBackend {
    const ENGINE: TvgEngine = TvgEngine::TvgEngineWg;
}

#[cfg(target_arch = "wasm32")]
impl WgBackend {
    pub fn new(threads: u32, html_canvas_selector: &str) -> Self {
        let context = em::WebGPUContext::new(html_canvas_selector);

        INSTANCES.init::<Self>(threads).unwrap();

        Self { context }
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for WgBackend {
    fn drop(&mut self) {
        INSTANCES.terminate::<Self>();
    }
}

#[cfg(target_arch = "wasm32")]
impl TvgBackend for WgBackend {
    fn create_canvas(&self) -> *mut tvg::Tvg_Canvas {
        unsafe { tvg::tvg_wgcanvas_create() }
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
        let device = self.context.device();
        let instance = self.context.instance();
        let surface = self.context.surface();

        if device.is_null() {
            return Err(TvgError::InvalidArgument);
        }

        unsafe {
            tvg::tvg_wgcanvas_set_target(
                canvas,
                device as *mut c_void,
                instance as *mut c_void,
                surface as *mut c_void,
                width,
                height,
                color_space.into(),
                0,
            )
            .into_result()
        }
    }
}
