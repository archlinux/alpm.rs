use crate::depends::satisfies_ver;
use alpm::{Alpm, Depend, Package, Result};

/// Extension methods to the [`Alpm`] type.
pub trait AlpmExt {
    /// Try to find a [`Package`] that satisfies a given dependency.
    fn find_local_satisfier<S>(&self, pkg: S) -> Result<Option<Package>>
    where
        S: Into<String>;
}

impl AlpmExt for Alpm {
    fn find_local_satisfier<S>(&self, pkg: S) -> Result<Option<Package>>
    where
        S: Into<String>,
    {
        let localdb = self.localdb();
        let pkg = pkg.into();

        if let Ok(alpm_pkg) = localdb.pkg(pkg.as_str()) {
            if satisfies_ver(Depend::new(pkg.as_str()), alpm_pkg.version()) {
                return Ok(Some(alpm_pkg));
            }
        }

        Ok(localdb.pkgs().find_satisfier(pkg))
    }
}
