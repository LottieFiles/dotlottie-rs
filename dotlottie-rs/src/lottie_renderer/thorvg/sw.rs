use super::common::{BackendInstances, TvgEngineInit};
use super::error::TvgError;
use super::types::*;
use super::*;

pub struct SwBackend;

static INSTANCES: BackendInstances = BackendInstances::new();

impl TvgEngineInit for SwBackend {
    const ENGINE: TvgEngine = TvgEngine::TvgEngineSw;
}

impl SwBackend {
    pub fn new(threads: u32) -> Self {
        INSTANCES.init::<Self>(threads).unwrap();

        Self {}
    }
}

impl Drop for SwBackend {
    fn drop(&mut self) {
        INSTANCES.terminate::<Self>();
    }
}

impl TvgBackend for SwBackend {
    fn create_canvas(&self) -> *mut tvg::Tvg_Canvas {
        unsafe { tvg::tvg_swcanvas_create() }
    }

    fn set_target(
        &self,
        canvas: *mut tvg::Tvg_Canvas,
        buffer: &mut Vec<u32>,
        stride: u32,
        width: u32,
        height: u32,
        color_space: ColorSpace,
    ) -> Result<(), TvgError> {
        unsafe {
            tvg::tvg_swcanvas_set_target(
                canvas,
                buffer.as_mut_ptr(),
                stride,
                width,
                height,
                color_space.into(),
            )
            .into_result()
        }
    }
}
