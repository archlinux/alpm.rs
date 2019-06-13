#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(not(feature = "git"))]
mod ffi;

#[cfg(feature = "git")]
mod ffi_git;

#[cfg(not(feature = "git"))]
pub use crate::ffi::*;

#[cfg(feature = "git")]
pub use crate::ffi_git::*;
