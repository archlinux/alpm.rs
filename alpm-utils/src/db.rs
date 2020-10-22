use alpm::{AlpmList, Db, Package, Result};

use crate::AsTarg;

/// Extention for AlpmList<Db>
pub trait DbListExt<'a> {
    /// Similar to find_satisfier() but expects a Target instead of a &str.
    fn find_target_satisfier<T: AsTarg>(&self, target: T) -> Option<Package<'a>>;
    /// Similar to pkg() but expects a Target instead of a &str.
    fn find_target<T: AsTarg>(&self, target: T) -> Option<Package<'a>>;
    /// The same as pkg() on Db but will try each Db in order return the first match.
    fn pkg<S: Into<String>>(&self, pkg: S) -> Result<Package<'a>>;
}

impl<'a> DbListExt<'a> for AlpmList<'a, Db<'a>> {
    fn find_target_satisfier<T: AsTarg>(&self, target: T) -> Option<Package<'a>> {
        let target = target.as_targ();

        if let Some(repo) = target.repo {
            if let Some(db) = self.iter().find(|r| r.name() == repo) {
                return db.pkgs().find_satisfier(target.pkg);
            }
        } else {
            return self.find_satisfier(target.pkg);
        }

        None
    }

    fn find_target<T: AsTarg>(&self, target: T) -> Option<Package<'a>> {
        let target = target.as_targ();

        if let Some(repo) = target.repo {
            if let Some(db) = self.iter().find(|r| r.name() == repo) {
                return db.pkg(target.pkg).ok();
            }
        } else {
            for db in self {
                if let Ok(pkg) = db.pkg(target.pkg) {
                    return Some(pkg);
                }
            }
        }

        None
    }

    fn pkg<S: Into<String>>(&self, pkg: S) -> Result<Package<'a>> {
        let pkg = pkg.into();

        for db in self {
            let pkg = db.pkg(&pkg);
            if pkg.is_ok() {
                return pkg;
            }
        }

        Err(alpm::Error::PkgNotFound)
    }
}
