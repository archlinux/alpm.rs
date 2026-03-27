use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
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

struct UnneededState<'a> {
    unneeded: Vec<&'a Package>,
    all_pkgs: HashMap<&'a str, &'a Package>,
    all_provides: HashMap<&'a str, Vec<(&'a Package, &'a Dep)>>,
}

fn find_unneeded_inner<'a>(handle: &'a Alpm, keep_optional: bool) -> UnneededState<'a> {
    let db = handle.localdb();

    let mut next = Vec::new();
    let mut deps: HashMap<&str, &Package> = HashMap::new();
    let mut all_pkgs: HashMap<&str, &Package> = HashMap::new();
    let mut all_provides: HashMap<&str, Vec<(&Package, &Dep)>> = HashMap::new();

    for pkg in db.pkgs().iter() {
        all_pkgs.insert(pkg.name(), pkg);
        for prov in pkg.provides() {
            all_provides
                .entry(prov.name())
                .or_default()
                .push((pkg, prov));
        }
        if pkg.reason() == PackageReason::Explicit {
            next.push(pkg);
        } else {
            deps.insert(pkg.name(), pkg);
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

                if let Some(provs) = all_provides.get(dep.name()) {
                    for &(prov_pkg, prov) in provs {
                        if satisfies_provide(dep, prov)
                            && let Some(removed) = deps.remove(prov_pkg.name())
                        {
                            next.push(removed);
                        }
                    }
                }
            }
        }
    }

    UnneededState {
        unneeded: deps.into_values().collect(),
        all_pkgs,
        all_provides,
    }
}

/// Find all recursively unneeded packages.
/// If `keep_optional` is true, optional dependencies are also followed.
pub fn find_unneeded(handle: &Alpm, keep_optional: bool) -> Vec<&Package> {
    find_unneeded_inner(handle, keep_optional).unneeded
}

/// An unneeded package with classification.
#[derive(Debug)]
pub struct UnneededPackage<'a> {
    /// The package.
    pub pkg: &'a Package,
    /// True if this is a direct orphan (no installed package depends on it).
    /// False if only other unneeded packages depend on it.
    pub direct: bool,
}

/// Like [`find_unneeded`], but classifies each result as a direct or indirect orphan.
pub fn find_unneeded_classified<'a>(
    handle: &'a Alpm,
    keep_optional: bool,
) -> Vec<UnneededPackage<'a>> {
    find_unneeded(handle, keep_optional)
        .into_iter()
        .map(|pkg| UnneededPackage {
            direct: is_orphan(pkg),
            pkg,
        })
        .collect()
}

/// Compute the minimal set of unneeded packages that must be removed together
/// with `targets` to avoid broken dependencies.
///
/// Given target package names to remove from the unneeded set, returns the
/// targets plus any unneeded packages whose hard dependencies would become
/// unsatisfied. Targets not in the unneeded set are silently ignored.
pub fn removal_closure<'a>(
    handle: &'a Alpm,
    targets: &[&str],
    keep_optional: bool,
) -> Vec<&'a Package> {
    let state = find_unneeded_inner(handle, keep_optional);

    let unneeded_set: HashSet<&str> = state.unneeded.iter().map(|p| p.name()).collect();

    let mut removal: HashSet<&str> = targets
        .iter()
        .copied()
        .filter(|t| unneeded_set.contains(t))
        .collect();

    let mut changed = true;
    while changed {
        changed = false;
        for pkg in &state.unneeded {
            if removal.contains(pkg.name()) {
                continue;
            }
            let has_broken_dep = pkg.depends().into_iter().any(|dep| {
                let direct_ok = state.all_pkgs.get(dep.name()).is_some_and(|p| {
                    !removal.contains(p.name())
                        && satisfies_dep(dep, p.name(), p.version())
                });
                if direct_ok {
                    return false;
                }
                let provide_ok =
                    state.all_provides.get(dep.name()).is_some_and(|provs| {
                        provs.iter().any(|(p, prov)| {
                            !removal.contains(p.name()) && satisfies_provide(dep, *prov)
                        })
                    });
                !provide_ok
            });
            if has_broken_dep {
                removal.insert(pkg.name());
                changed = true;
            }
        }
    }

    state
        .unneeded
        .into_iter()
        .filter(|p| removal.contains(p.name()))
        .collect()
}

/// Orphan and unneeded package detection for [`Alpm`].
pub trait OrphanExt {
    /// Find all direct orphan packages. See [`find_unneeded`] for recursive detection.
    fn find_orphans(&self) -> impl Iterator<Item = &Package>;
    /// See [`find_unneeded`].
    fn find_unneeded(&self, keep_optional: bool) -> Vec<&Package>;
    /// See [`find_unneeded_classified`].
    fn find_unneeded_classified(&self, keep_optional: bool) -> Vec<UnneededPackage<'_>>;
    /// See [`removal_closure`].
    fn removal_closure(&self, targets: &[&str], keep_optional: bool) -> Vec<&Package>;
}

impl OrphanExt for Alpm {
    fn find_orphans(&self) -> impl Iterator<Item = &Package> {
        self.localdb().pkgs().iter().filter(|pkg| is_orphan(pkg))
    }

    fn find_unneeded(&self, keep_optional: bool) -> Vec<&Package> {
        find_unneeded(self, keep_optional)
    }

    fn find_unneeded_classified(&self, keep_optional: bool) -> Vec<UnneededPackage<'_>> {
        find_unneeded_classified(self, keep_optional)
    }

    fn removal_closure(&self, targets: &[&str], keep_optional: bool) -> Vec<&Package> {
        removal_closure(self, targets, keep_optional)
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
        assert_eq!(names, ["orphan-a", "orphan-consumer"]);
    }

    #[test]
    fn test_find_unneeded() {
        let handle = test_handle();
        let mut names = unneeded_names(&handle, false);
        names.sort();
        assert_eq!(
            names,
            [
                "opt-dep",
                "orphan-a",
                "orphan-b",
                "orphan-c",
                "orphan-consumer",
                "orphan-provider"
            ]
        );
    }

    #[test]
    fn test_find_unneeded_keep_optional() {
        let handle = test_handle();
        let mut names = unneeded_names(&handle, true);
        names.sort();
        assert_eq!(
            names,
            [
                "orphan-a",
                "orphan-b",
                "orphan-c",
                "orphan-consumer",
                "orphan-provider"
            ]
        );
    }

    fn closure_names<'a>(handle: &'a Alpm, targets: &[&str]) -> Vec<&'a str> {
        let mut names: Vec<_> = removal_closure(handle, targets, true)
            .iter()
            .map(|p| p.name())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn test_removal_closure_leaf() {
        let handle = test_handle();
        assert_eq!(closure_names(&handle, &["orphan-c"]), ["orphan-a", "orphan-b", "orphan-c"]);
    }

    #[test]
    fn test_removal_closure_root() {
        let handle = test_handle();
        assert_eq!(closure_names(&handle, &["orphan-a"]), ["orphan-a"]);
    }

    #[test]
    fn test_removal_closure_middle() {
        let handle = test_handle();
        assert_eq!(closure_names(&handle, &["orphan-b"]), ["orphan-a", "orphan-b"]);
    }

    #[test]
    fn test_removal_closure_empty() {
        let handle = test_handle();
        assert!(closure_names(&handle, &[]).is_empty());
    }

    #[test]
    fn test_removal_closure_not_unneeded() {
        let handle = test_handle();
        assert!(closure_names(&handle, &["needed-lib"]).is_empty());
    }

    #[test]
    fn test_removal_closure_multiple_targets() {
        let handle = test_handle();
        assert_eq!(
            closure_names(&handle, &["orphan-a", "orphan-provider"]),
            ["orphan-a", "orphan-consumer", "orphan-provider"]
        );
    }

    #[test]
    fn test_removal_closure_keep_optional_false() {
        let handle = test_handle();
        let mut names: Vec<_> = removal_closure(&handle, &["orphan-c"], false)
            .iter()
            .map(|p| p.name())
            .collect();
        names.sort();
        assert_eq!(names, ["orphan-a", "orphan-b", "orphan-c"]);
    }

    #[test]
    fn test_removal_closure_virtual_dep() {
        let handle = test_handle();
        assert_eq!(
            closure_names(&handle, &["orphan-provider"]),
            ["orphan-consumer", "orphan-provider"]
        );
    }

    #[test]
    fn test_find_unneeded_classified() {
        let handle = test_handle();
        let mut classified = find_unneeded_classified(&handle, true);
        classified.sort_by_key(|u| u.pkg.name());

        let result: Vec<_> = classified
            .iter()
            .map(|u| (u.pkg.name(), u.direct))
            .collect();
        assert_eq!(
            result,
            [
                ("orphan-a", true),
                ("orphan-b", false),
                ("orphan-c", false),
                ("orphan-consumer", true),
                ("orphan-provider", false),
            ]
        );
    }
}
