//! #alpm-utils
//!
//! A utility libary that provides some common functionality an alpm user may requre.

#![warn(missing_docs)]

mod db;
/// Utils for dependency checking.
pub mod depends;
mod target;

pub use crate::db::*;
pub use crate::target::*;
