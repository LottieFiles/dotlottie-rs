#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "uniffi")]
mod uniffi_impl;

#[cfg(feature = "uniffi")]
pub use uniffi_impl::*;

#[cfg(feature = "uniffi")]
uniffi::include_scaffolding!("dotlottie_player");
