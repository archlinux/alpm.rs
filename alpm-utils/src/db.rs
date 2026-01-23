use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::take;

use alpm::{Alpm, AlpmList, AlpmListMut, Db, Dep, Package, PackageReason, Result};

use crate::depends::{satisfies_dep, satisfies_provide};
use crate::AsTarg;

/// Check if a package is a direct orphan.
pub fn is_orphan(pkg: &Package) -> bool {
    pkg.reason() == PackageReason::Depend
        && pkg.required_by().is_empty()
        && pkg.optional_for().is_empty()
}

/// Find all recursively unneeded packages
/// If `keep_optional` is true, optional dependencies are also followed.
pub fn find_unneeded(handle: &Alpm, keep_optional: bool) -> Vec<&Package> {
    let db = handle.localdb();

    let mut next: Vec<&Package> = db
        .pkgs()
        .iter()
        .filter(|p| p.reason() == PackageReason::Explicit)
        .collect();

    let mut deps: HashMap<&str, &Package> = db
        .pkgs()
        .iter()
        .filter(|p| p.reason() != PackageReason::Explicit)
        .map(|p| (p.name(), p))
        .collect();

    let mut provides: HashMap<&str, Vec<(&Package, &Dep)>> = HashMap::new();
    for pkg in deps.values() {
        for prov in pkg.provides() {
            provides
                .entry(prov.name())
                .or_default()
                .push((*pkg, prov));
        }
    }

    while !next.is_empty() {
        for pkg in take(&mut next) {
            let opt = keep_optional.then(|| pkg.optdepends());
            let depends = pkg.depends().into_iter().chain(opt.into_iter().flatten());

            for dep in depends {
                if let Entry::Occupied(entry) = deps.entry(dep.name()) {
                    let candidate = entry.get();
                    if satisfies_dep(dep, candidate.name(), candidate.version()) {
                        next.push(entry.remove());
                    }
                }

                if let Entry::Occupied(mut entry) = provides.entry(dep.name()) {
                    let found: Vec<&Package> = entry
                        .get_mut()
                        .extract_if(.., |(_, prov)| satisfies_provide(dep, *prov))
                        .filter_map(|(pkg, _)| deps.remove(pkg.name()))
                        .collect();
                    next.extend(found);

                    if entry.get().is_empty() {
                        entry.remove();
                    }
                }
            }
        }
    }

    deps.into_values().collect()
}

/// Orphan and unneeded package detection for [`Alpm`].
pub trait OrphanExt {
    /// Find all direct orphan packages. See [`find_unneeded`] for recursive detection.
    fn find_orphans(&self) -> impl Iterator<Item = &Package>;
    /// See [`find_unneeded`].
    fn find_unneeded(&self, keep_optional: bool) -> Vec<&Package>;
}

impl OrphanExt for Alpm {
    fn find_orphans(&self) -> impl Iterator<Item = &Package> {
        self.localdb().pkgs().iter().filter(|pkg| is_orphan(pkg))
    }

    fn find_unneeded(&self, keep_optional: bool) -> Vec<&Package> {
        find_unneeded(self, keep_optional)
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
        Alpm::new("/", "../alpm/tests/unneeded_db").unwrap()
    }

    fn unneeded_names(handle: &Alpm, keep_optional: bool) -> Vec<&str> {
        find_unneeded(handle, keep_optional)
            .iter()
            .map(|p| p.name())
            .collect()
    }

    #[test]
    fn test_is_orphan() {
        let handle = test_handle();
        let db = handle.localdb();
        assert!(!is_orphan(db.pkg("needed-lib").unwrap()));
        assert!(is_orphan(db.pkg("orphan-a").unwrap()));
        assert!(!is_orphan(db.pkg("explicit-app").unwrap()));
    }

    #[test]
    fn test_find_orphans() {
        let handle = test_handle();
        let mut names: Vec<_> = handle.find_orphans().map(|p| p.name()).collect();
        names.sort();
        assert_eq!(names, ["orphan-a"]);
    }

    #[test]
    fn test_find_unneeded() {
        let handle = test_handle();
        let mut names = unneeded_names(&handle, false);
        names.sort();
        assert_eq!(names, ["opt-dep", "orphan-a", "orphan-b", "orphan-c"]);
    }

    #[test]
    fn test_find_unneeded_keep_optional() {
        let handle = test_handle();
        let mut names = unneeded_names(&handle, true);
        names.sort();
        assert_eq!(names, ["orphan-a", "orphan-b", "orphan-c"]);
    }
}
