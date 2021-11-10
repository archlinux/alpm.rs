//! # Alpm
//!
//! See [`Alpm`] as the base type to interact with alpm.
//!
#![doc = include_str!("../../README.md")]

mod add;
mod alpm;
mod be_local;
mod be_pkg;
mod be_sync;
mod cb;
mod conflict;
mod db;
mod deps;
mod dload;
mod error;
mod filelist;
mod handle;
mod list;
mod log;
#[cfg(feature = "mtree")]
mod mtree;
mod package;
mod remove;
mod signing;
mod sync;
mod trans;
mod types;
mod util;
mod utils;

mod version;

pub use crate::add::*;
pub use crate::alpm::*;
pub use crate::be_local::*;
pub use crate::be_pkg::*;
pub use crate::be_sync::*;
pub use crate::cb::*;
pub use crate::conflict::*;
pub use crate::db::*;
pub use crate::deps::*;
pub use crate::dload::*;
pub use crate::error::*;
pub use crate::filelist::*;
pub use crate::handle::*;
pub use crate::list::*;
#[cfg(feature = "mtree")]
pub use crate::mtree::*;
pub use crate::package::*;
pub use crate::remove::*;
pub use crate::signing::*;
pub use crate::sync::*;
pub use crate::trans::*;
pub use crate::types::*;
pub use crate::util::*;
pub use crate::version::*;
