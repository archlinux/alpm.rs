#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(not(any(feature = "generate", alpm16)))]
mod ffi;

#[cfg(all(alpm16, not(feature = "generate")))]
mod ffi_git;

#[cfg(feature = "generate")]
mod ffi_generated;

#[cfg(not(any(feature = "generate", alpm16)))]
pub use crate::ffi::*;

#[cfg(all(alpm16, not(feature = "generate")))]
pub use crate::ffi_git::*;

#[cfg(feature = "generate")]
pub use crate::ffi_generated::*;
