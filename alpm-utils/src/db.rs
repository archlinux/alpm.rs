use alpm::{AlpmList, Db, Package, Result};

use crate::Target;

/// Extention for AlpmList<Db>
pub trait DbListExt<'a> {
    /// Similar to find_satisfier() but expects a Target instead of a &str.
    fn find_target_satisfier<'b, T: Into<Target<'b>>>(
        &mut self,
        target: T,
    ) -> Result<Option<Package<'a>>>;
    /// Similar to pkg() but expects a Target instead of a &str.
    fn find_target<'b, T: Into<Target<'b>>>(&mut self, target: T) -> Option<Package<'a>>;
    /// The same as pkg() on Db but will try each Db in order return the first match.
    fn pkg<S: Into<String>>(&mut self, pkg: S) -> Result<Package<'_>>;
}

impl<'a> DbListExt<'a> for AlpmList<'a, Db<'a>> {
    fn find_target_satisfier<'b, T: Into<Target<'b>>>(
        &mut self,
        target: T,
    ) -> Result<Option<Package<'a>>> {
        let target = target.into();

        if let Some(repo) = target.repo {
            if let Some(db) = self.find(|r| r.name() == repo) {
                return Ok(db.pkgs()?.find_satisfier(target.pkg));
            }
        } else {
            return Ok(self.find_satisfier(target.pkg));
        }

        Ok(None)
    }

    fn find_target<'b, T: Into<Target<'b>>>(&mut self, target: T) -> Option<Package<'a>> {
        let target = target.into();

        if let Some(repo) = target.repo {
            if let Some(db) = self.find(|r| r.name() == repo) {
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

    fn pkg<S: Into<String>>(&mut self, pkg: S) -> Result<Package<'_>> {
        let pkg = pkg.into();

        for db in self {
            let pkg = db.pkg(&pkg);
            if pkg.is_ok() {
                return pkg;
            }
        }

        return Err(alpm::Error::PkgNotFound);
    }
}
