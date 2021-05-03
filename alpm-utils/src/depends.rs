use alpm::{AsDep, DepModVer, Ver};

/// Checks if a dependency is satisfied by a package (name + version).
pub fn satisfies_dep<'a, S: AsRef<str>, V: AsRef<Ver>>(
    dep: impl AsDep,
    name: S,
    version: V,
) -> bool {
    let name = name.as_ref();
    let dep = dep.as_dep();

    if dep.name() != name {
        return false;
    }

    satisfies_ver(dep, version)
}

/// Checks if a dependency is satisdied by a provide.
pub fn satisfies_provide(dep: impl AsDep, provide: impl AsDep) -> bool {
    let dep = dep.as_dep();
    let provide = provide.as_dep();

    if dep.name() != provide.name() {
        return false;
    }

    if provide.version().is_none() && dep.version().is_some() {
        return false;
    }

    match provide.version() {
        None => dep.version().is_none(),
        Some(ver) => satisfies_ver(dep, ver),
    }
}

/// Checks if a Dep is satisfied by a name + version + provides combo
pub fn satisfies<'a, D: AsDep, S: AsRef<str>, V: AsRef<Ver>>(
    dep: impl AsDep,
    name: S,
    version: V,
    mut provides: impl Iterator<Item = D>,
) -> bool {
    satisfies_dep(dep.as_dep(), name, version)
        || provides.any(|p| satisfies_provide(dep.as_dep(), p))
}

/// Checks if a Dep is satisfied by a name + provides (ignoring version) combo
pub fn satisfies_nover<'a, D: AsDep, S: AsRef<str>>(
    dep: impl AsDep,
    name: S,
    mut provides: impl Iterator<Item = D>,
) -> bool {
    satisfies_dep_nover(dep.as_dep(), name)
        || provides.any(|p| satisfies_provide_nover(dep.as_dep(), p))
}

/// Checks if a dependency is satisfied by a package (name only).
pub fn satisfies_dep_nover<'a, S: AsRef<str>>(dep: impl AsDep, name: S) -> bool {
    dep.as_dep().name() == name.as_ref()
}

/// Checks if a dependency is satisdied by a provide (name only).
pub fn satisfies_provide_nover(dep: impl AsDep, provide: impl AsDep) -> bool {
    dep.as_dep().name() == provide.as_dep().name()
}

fn satisfies_ver<V: AsRef<Ver>>(dep: impl AsDep, version: V) -> bool {
    let version = version.as_ref();
    let dep = dep.as_dep();

    match dep.depmodver() {
        DepModVer::Any => true,
        DepModVer::Eq(dep) => version == dep || version.split('-').next().unwrap() == dep.as_str(),
        DepModVer::Ge(dep) => version >= dep,
        DepModVer::Le(dep) => version <= dep,
        DepModVer::Gt(dep) => version > dep,
        DepModVer::Lt(dep) => version < dep,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alpm::{Depend, Version};

    #[test]
    fn test_satisfies_ver() {
        assert!(satisfies_ver(
            Depend::new("foo>0").as_dep(),
            Version::new("9.0.0")
        ));
        assert!(satisfies_ver(Depend::new("foo<10"), Version::new("9.0.0")));
        assert!(satisfies_ver(Depend::new("foo=10"), Version::new("10")));
        assert!(satisfies_ver(Depend::new("foo=10-1"), Version::new("10-1")));
        assert!(satisfies_ver(Depend::new("foo=10"), Version::new("10-1")));
        assert!(satisfies_ver(Depend::new("foo=10"), Version::new("10-2")));
        assert!(!satisfies_ver(
            Depend::new("foo=10-1"),
            Version::new("10-2")
        ));
        assert!(satisfies_ver(Depend::new("foo<=10"), Version::new("9.0.0")));
        assert!(satisfies_ver(Depend::new("foo>=9"), Version::new("9.0.0")));
        assert!(satisfies_ver(
            Depend::new("foo=9.0.0"),
            Version::new("9.0.0")
        ));

        assert!(!satisfies_ver(
            Depend::new("foo>=10"),
            Version::new("9.0.0")
        ));
        assert!(!satisfies_ver(Depend::new("foo<=8"), Version::new("9.0.0")));
        assert!(!satisfies_ver(Depend::new("foo=8"), Version::new("9.0.0")));

        assert!(satisfies_ver(Depend::new("foo"), Version::new("1")));
        assert!(satisfies_ver(Depend::new("foo"), Version::new("1.0.0")));
    }
}
