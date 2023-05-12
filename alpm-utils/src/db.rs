use alpm::{AlpmList, Db, Package, Result};

use crate::AsTarg;

/// Extentions for `AlpmList<Db>`.
pub trait DbListExt<'a> {
    /// Similar to find_satisfier() but expects a Target instead of a &str.
    fn find_target_satisfier<T: AsTarg>(&self, target: T) -> Option<Package<'a>>;
    /// Similar to pkg() but expects a Target instead of a &str.
    fn find_target<T: AsTarg>(&self, target: T) -> Result<Package<'a>>;
    /// The same as pkg() on Db but will try each Db in order return the first match.
    fn pkg<S: Into<Vec<u8>>>(&self, pkg: S) -> Result<Package<'a>>;
}

impl<'a> DbListExt<'a> for AlpmList<'a, Db<'a>> {
    fn find_target_satisfier<T: AsTarg>(&self, target: T) -> Option<Package<'a>> {
        let target = target.as_targ();

        if let Some(repo) = target.repo {
            if let Some(db) = self.iter().find(|r| r.name() == repo) {
                db.pkgs().find_satisfier(target.pkg)
            } else {
                None
            }
        } else {
            self.find_satisfier(target.pkg)
        }
    }

    fn find_target<T: AsTarg>(&self, target: T) -> Result<Package<'a>> {
        let target = target.as_targ();

        if let Some(repo) = target.repo {
            if let Some(db) = self.iter().find(|r| r.name() == repo) {
                db.pkg(target.pkg)
            } else {
                Err(alpm::Error::PkgNotFound)
            }
        } else {
            self.pkg(target.pkg)
        }
    }

    fn pkg<S: Into<Vec<u8>>>(&self, pkg: S) -> Result<Package<'a>> {
        let mut pkg = pkg.into();
        pkg.reserve(1);
        let pkg = self.iter().find_map(|db| db.pkg(pkg.clone()).ok());
        pkg.ok_or(alpm::Error::PkgNotFound)
    }
}
