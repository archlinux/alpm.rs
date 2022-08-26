//! Extension methods for the [`Alpm`] type.

use crate::DbListExt;
use alpm::{Alpm, Package};

/// All official packages.
pub fn native_packages(alpm: &Alpm) -> impl Iterator<Item = Package<'_>> {
    let syncs = alpm.syncdbs();

    alpm.localdb()
        .pkgs()
        .into_iter()
        .filter_map(move |p| syncs.pkg(p.name()).ok())
}

/// All foreign packages as an `Iterator`.
pub fn foreign_packages(alpm: &Alpm) -> impl Iterator<Item = Package<'_>> {
    let syncs = alpm.syncdbs();

    alpm.localdb()
        .pkgs()
        .into_iter()
        .filter(move |p| syncs.pkg(p.name()).is_err())
}
