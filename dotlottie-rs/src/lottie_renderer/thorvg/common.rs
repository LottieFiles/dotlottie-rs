use super::types::*;
use super::*;
use std::sync::Mutex;

pub(crate) trait TvgEngineInit {
    const ENGINE: TvgEngine;

    fn init_engine(threads: u32) -> Result<(), TvgError> {
        unsafe { tvg::tvg_engine_init(Self::ENGINE.into(), threads).into_result() }
    }

    fn term_engine() {
        unsafe {
            tvg::tvg_engine_term(Self::ENGINE.into());
        }
    }
}

pub(crate) struct BackendInstances(Mutex<usize>);

impl BackendInstances {
    pub const fn new() -> Self {
        Self(Mutex::new(0))
    }

    pub fn init<T: TvgEngineInit>(&self, threads: u32) -> Result<(), TvgError> {
        let mut count = self.0.lock().map_err(|_| TvgError::Unknown)?;

        if *count == 0 {
            T::init_engine(threads)?;
        }

        *count += 1;
        Ok(())
    }

    pub fn terminate<T: TvgEngineInit>(&self) {
        let mut count = self.0.lock().unwrap();

        *count = count.checked_sub(1).unwrap();
        if *count == 0 {
            T::term_engine();
        }
    }
}
