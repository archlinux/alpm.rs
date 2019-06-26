use alpm::{vercmp, DepMod, Depend};

use std::cmp::Ordering;

/// Checks if a dependency is satisfied by a package (name + version).
pub fn satisfies_dep<'a, D: Into<Depend<'a>>, S: AsRef<str>>(dep: D, name: S, version: S) -> bool {
    let dep = dep.into();
    let name = name.as_ref();
    let version = version.as_ref();

    if dep.name() != name {
        return false;
    }

    satisfies_ver(dep, version)
}

/// Checks if a dependency is satisdied by a provide.
pub fn satisfies_provide<'a, D: Into<Depend<'a>>>(dep: D, provide: D) -> bool {
    let dep = dep.into();
    let provide = provide.into();

    if dep.name() != provide.name() {
        return false;
    }

    if provide.depmod() == DepMod::Any && dep.depmod() != DepMod::Any {
        return false;
    }

    satisfies_ver(dep, provide.version())
}

fn satisfies_ver<'a, D: Into<Depend<'a>>, S: AsRef<str>>(dep: D, version: S) -> bool {
    let dep = dep.into();
    let version = version.as_ref();

    if dep.depmod() == DepMod::Any {
        return true;
    }

    let cmp = vercmp(dep.version(), version);

    match dep.depmod() {
        DepMod::Eq => cmp == Ordering::Equal,
        DepMod::Ge => cmp == Ordering::Greater || cmp == Ordering::Equal,
        DepMod::Le => cmp == Ordering::Less || cmp == Ordering::Equal,
        DepMod::Gt => cmp == Ordering::Greater,
        DepMod::Lt => cmp == Ordering::Less,
        DepMod::Any => true,
    }
}
