#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(not(any(feature = "generate", feature = "git")))]
mod ffi;

#[cfg(all(feature = "git", not(feature = "generate")))]
mod ffi_git;

#[cfg(feature = "generate")]
mod ffi_generated;

#[cfg(not(any(feature = "generate", feature = "git")))]
pub use crate::ffi::*;

#[cfg(all(feature = "git", not(feature = "generate")))]
pub use crate::ffi_git::*;

#[cfg(feature = "generate")]
pub use crate::ffi_generated::*;
