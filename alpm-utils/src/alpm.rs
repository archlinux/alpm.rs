use crate::depends::satisfies_ver;

use alpm::{Alpm, Result, Package, Depend};

pub trait AlpmExt {
    fn find_local_satisfier(&self, pkg: impl AsRef<str>) -> Result<Option<Package>>;
}

impl AlpmExt for Alpm {
    fn find_local_satisfier(&self, pkg: impl AsRef<str>) -> Result<Option<Package>> {
        let localdb = self.localdb();
        let pkg = pkg.as_ref();

        if let Ok(alpm_pkg) = localdb.pkg(&pkg) {
            if satisfies_ver(&Depend::new(&pkg), alpm_pkg.version()) {
                return Ok(Some(alpm_pkg));
            }
        }

        return Ok(localdb.pkgs()?.find_satisfier(pkg));
    }
}
