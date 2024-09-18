//! # Alpm
//!
//! See [`Alpm`] as the base type to interact with alpm.
//!
#![doc = include_str!("../README.md")]

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
mod list_mut;
mod list_with;
mod log;
#[cfg(feature = "mtree")]
mod mtree;
mod package;
mod remove;
mod sandbox;
mod signing;
mod sync;
mod trans;
mod types;
mod unions;
mod util;
mod utils;
mod version;

pub use crate::add::*;
pub use crate::alpm::*;
pub use crate::be_pkg::*;
pub use crate::cb::*;
pub use crate::conflict::*;
pub use crate::db::*;
pub use crate::deps::*;
pub use crate::error::*;
pub use crate::filelist::*;
pub use crate::list::*;
pub use crate::list_mut::*;
pub use crate::list_with::*;
#[cfg(feature = "mtree")]
pub use crate::mtree::*;
pub use crate::package::*;
pub use crate::signing::*;
pub use crate::trans::*;
pub use crate::types::*;
pub use crate::unions::*;
pub use crate::util::*;
pub use crate::version::*;
