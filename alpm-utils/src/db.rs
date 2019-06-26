use alpm::{AlpmList, Db, Package, Result};

use crate::Target;

/// Extention for AlpmList<Db>
pub trait DbListExt {
    /// Similar to find_satisfier but expects a Target instead of a &str.
    fn find_target<'a, T: Into<Target<'a>>>(&mut self, target: T) -> Result<Option<Package>>;
}

impl<'a> DbListExt for AlpmList<'a, Db<'a>> {
    fn find_target<'b, T: Into<Target<'b>>>(&mut self, target: T) -> Result<Option<Package>> {
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
}
