//! # alpm-utils
//!
//! A utility libary that provides some common functionality an alpm user may requre.

#![warn(missing_docs)]

#[cfg(feature = "conf")]
mod conf;
#[cfg(feature = "alpm")]
mod db;
/// Utils for dependency checking.
#[cfg(feature = "alpm")]
pub mod depends;
mod target;

#[cfg(feature = "conf")]
pub use crate::conf::*;
#[cfg(feature = "alpm")]
pub use crate::db::*;
pub use crate::target::*;
