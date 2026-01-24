use alpm::{Alpm, AlpmList, AlpmListMut, Db, Package, PackageReason, Result};

use crate::AsTarg;

/// Check if a package is an orphan.
pub fn is_orphan(pkg: &Package) -> bool {
    pkg.reason() == PackageReason::Depend
        && pkg.required_by().is_empty()
        && pkg.optional_for().is_empty()
}

/// Extension trait for Alpm providing orphan detection.
pub trait OrphanExt {
    /// Find all orphan packages in the local database.
    fn find_orphans(&self) -> impl Iterator<Item = &Package>;
}

impl OrphanExt for Alpm {
    fn find_orphans(&self) -> impl Iterator<Item = &Package> {
        self.localdb().pkgs().iter().filter(|pkg| is_orphan(pkg))
    }
}

/// Extension trait for `AlpmList<Db>`.
pub trait DbListExt<'a> {
    /// Similar to find_satisfier() but expects a Target instead of a &str.
    fn find_target_satisfier<T: AsTarg>(&self, target: T) -> Option<&'a Package>;
    /// Similar to pkg() but expects a Target instead of a &str.
    fn find_target<T: AsTarg>(&self, target: T) -> Result<&'a Package>;
    /// The same as pkg() on Db but will try each Db in order return the first match.
    fn pkg<S: Into<Vec<u8>>>(&self, pkg: S) -> Result<&'a Package>;
}

impl<'a> DbListExt<'a> for AlpmListMut<&'a Db> {
    fn find_target_satisfier<T: AsTarg>(&self, target: T) -> Option<&'a Package> {
        self.list().find_target_satisfier(target)
    }

    fn find_target<T: AsTarg>(&self, target: T) -> Result<&'a Package> {
        self.list().find_target(target)
    }

    fn pkg<S: Into<Vec<u8>>>(&self, pkg: S) -> Result<&'a Package> {
        self.list().pkg(pkg)
    }
}

impl<'a> DbListExt<'a> for AlpmList<'_, &'a Db> {
    fn find_target_satisfier<T: AsTarg>(&self, target: T) -> Option<&'a Package> {
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

    fn find_target<T: AsTarg>(&self, target: T) -> Result<&'a Package> {
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

    fn pkg<S: Into<Vec<u8>>>(&self, pkg: S) -> Result<&'a Package> {
        let mut pkg = pkg.into();
        pkg.reserve(1);
        let pkg = self.iter().find_map(|db| db.pkg(pkg.clone()).ok());
        pkg.ok_or(alpm::Error::PkgNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_handle() -> Alpm {
        Alpm::new("/", "../alpm/tests/db").unwrap()
    }

    #[test]
    fn test_is_orphan_dependency_with_no_dependents() {
        let handle = test_handle();
        let localdb = handle.localdb();

        // argon2 is installed as dependency (REASON=1) and has no required_by in test db
        let pkg = localdb.pkg("argon2").unwrap();
        assert_eq!(pkg.reason(), PackageReason::Depend);

        // Whether it's an orphan depends on if anything requires it
        let result = is_orphan(&pkg);
        // The function should return true if no packages require or optionally depend on it
        assert!(result == (pkg.required_by().is_empty() && pkg.optional_for().is_empty()));
    }

    #[test]
    fn test_is_orphan_explicit_package() {
        let handle = test_handle();
        let localdb = handle.localdb();

        // Find a package that's explicitly installed (if any in test db)
        for pkg in localdb.pkgs().iter() {
            if pkg.reason() == PackageReason::Explicit {
                // Explicit packages should never be orphans
                assert!(!is_orphan(&pkg));
                return;
            }
        }
    }

    #[test]
    fn test_find_orphans_iterator() {
        let handle = test_handle();

        // find_orphans should return an iterator
        let orphans: Vec<_> = handle.find_orphans().collect();

        // All returned packages should satisfy is_orphan
        for pkg in &orphans {
            assert!(is_orphan(pkg));
        }
    }
}
