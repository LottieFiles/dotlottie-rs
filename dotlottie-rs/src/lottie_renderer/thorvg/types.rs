use super::error::TvgError;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TvgEngine {
    TvgEngineSw,
    #[cfg(feature = "thorvg_v1")]
    TvgEngineGl,
    #[cfg(feature = "thorvg_v1")]
    TvgEngineWg,
}

impl From<TvgEngine> for tvg::Tvg_Engine {
    fn from(engine: TvgEngine) -> Self {
        match engine {
            TvgEngine::TvgEngineSw => tvg::Tvg_Engine_TVG_ENGINE_SW,
            #[cfg(feature = "thorvg_v1")]
            TvgEngine::TvgEngineGl => tvg::Tvg_Engine_TVG_ENGINE_GL,
            #[cfg(feature = "thorvg_v1")]
            TvgEngine::TvgEngineWg => tvg::Tvg_Engine_TVG_ENGINE_WG,
            #[cfg(not(feature = "thorvg_v1"))]
            _ => unreachable!(),
        }
    }
}

pub trait TvgBackend {
    fn create_canvas(&self) -> *mut tvg::Tvg_Canvas;
    fn set_target(
        &self,
        canvas: *mut tvg::Tvg_Canvas,
        buffer: &mut Vec<u32>,
        stride: u32,
        width: u32,
        height: u32,
        color_space: ColorSpace,
    ) -> Result<(), TvgError>;
}

impl From<ColorSpace> for tvg::Tvg_Colorspace {
    fn from(color_space: ColorSpace) -> Self {
        match color_space {
            ColorSpace::ABGR8888 => tvg::Tvg_Colorspace_TVG_COLORSPACE_ABGR8888,
            ColorSpace::ABGR8888S => tvg::Tvg_Colorspace_TVG_COLORSPACE_ABGR8888S,
            ColorSpace::ARGB8888 => tvg::Tvg_Colorspace_TVG_COLORSPACE_ARGB8888,
            ColorSpace::ARGB8888S => tvg::Tvg_Colorspace_TVG_COLORSPACE_ARGB8888S,
        }
    }
}
