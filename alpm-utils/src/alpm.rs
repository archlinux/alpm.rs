//! Extension methods for the [`Alpm`] type.

use crate::DbListExt;
use alpm::{Alpm, AlpmList, Db, IntoIter, Package};

/// Extension methods for [`Alpm`] which aren't critical enough to live in the
/// main crate, directly within `Alpm`.
pub trait AlpmExt {
    /// An iterator of [`Package`]s that are found in "sync databases",
    /// typically registered in one's `pacman.conf`.
    fn native_packages<'a>(&'a self) -> NativePkgs<'a>;

    /// The opposite of [`AlpmExt::native_packages`]; installed packages that
    /// aren't found in any registered "sync database".
    fn foreign_packages<'a>(&'a self) -> ForeignPkgs<'a>;
}

impl AlpmExt for Alpm {
    fn native_packages<'a>(&'a self) -> NativePkgs<'a> {
        NativePkgs::new(self)
    }

    fn foreign_packages<'a>(&'a self) -> ForeignPkgs<'a> {
        ForeignPkgs::new(self)
    }
}

/// [`Package`]s that are found in registered "sync databases".
pub struct NativePkgs<'a> {
    local: IntoIter<'a, Package<'a>>,
    sync: AlpmList<'a, Db<'a>>,
}

impl<'a> NativePkgs<'a> {
    fn new(alpm: &'a Alpm) -> NativePkgs<'a> {
        let local = alpm.localdb().pkgs().into_iter();
        let sync = alpm.syncdbs();

        NativePkgs { local, sync }
    }
}

impl<'a> Iterator for NativePkgs<'a> {
    type Item = Package<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let s = self.sync;
        self.local.find(|p| s.pkg(p.name()).is_ok())
    }
}

/// Installed [`Package`]s that are _not_ found in registered "sync databases".
pub struct ForeignPkgs<'a> {
    local: IntoIter<'a, Package<'a>>,
    sync: AlpmList<'a, Db<'a>>,
}

impl<'a> ForeignPkgs<'a> {
    fn new(alpm: &'a Alpm) -> ForeignPkgs<'a> {
        let local = alpm.localdb().pkgs().into_iter();
        let sync = alpm.syncdbs();

        ForeignPkgs { local, sync }
    }
}

impl<'a> Iterator for ForeignPkgs<'a> {
    type Item = Package<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let s = self.sync;
        self.local.find(|p| s.pkg(p.name()).is_err())
    }
}
