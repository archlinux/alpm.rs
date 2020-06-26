use alpm::{DepModVer, Depend, Ver};

/// Checks if a dependency is satisfied by a package (name + version).
pub fn satisfies_dep<'a, S: AsRef<str>, V: AsRef<Ver>>(dep: &Depend, name: S, version: V) -> bool {
    let name = name.as_ref();

    if dep.name() != name {
        return false;
    }

    satisfies_ver(dep, version)
}

/// Checks if a dependency is satisdied by a provide.
pub fn satisfies_provide<'a>(dep: &Depend<'a>, provide: &Depend<'a>) -> bool {
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

/// Checks if a Depend is satisfied by a name + version + provides combo
pub fn satisfies<'a, S: AsRef<str>, V: AsRef<Ver>>(
    dep: &Depend,
    name: S,
    version: V,
    mut provides: impl Iterator<Item = Depend<'a>>,
) -> bool {
    satisfies_dep(dep, name, version) || provides.any(|p| p.name() == dep.name())
}

/// Checks if a Depend is satisfied by a name + provides (ignoring version) combo
pub fn satisfies_nover<'a, S: AsRef<str>>(
    dep: &Depend,
    name: S,
    mut provides: impl Iterator<Item = Depend<'a>>,
) -> bool {
    satisfies_dep_nover(dep, name) || provides.any(|p| p.name() == dep.name())
}

/// Checks if a dependency is satisfied by a package (name only).
pub fn satisfies_dep_nover<'a, S: AsRef<str>>(dep: &Depend, name: S) -> bool {
    dep.name() == name.as_ref()
}

/// Checks if a dependency is satisdied by a provide (name only).
pub fn satisfies_provide_nover<'a>(dep: &Depend<'a>, provide: &Depend<'a>) -> bool {
    dep.name() == provide.name()
}

fn satisfies_ver<'a, V: AsRef<Ver>>(dep: &Depend<'a>, version: V) -> bool {
    let version = version.as_ref();

    match dep.depmodver() {
        DepModVer::Any => true,
        DepModVer::Eq(dep) => version == dep,
        DepModVer::Ge(dep) => version >= dep,
        DepModVer::Le(dep) => version <= dep,
        DepModVer::Gt(dep) => version > dep,
        DepModVer::Lt(dep) => version < dep,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alpm::Version;

    #[test]
    fn test_satisfies_ver() {
        assert!(satisfies_ver(&Depend::new("foo>0"), Version::new("9.0.0")));
        assert!(satisfies_ver(&Depend::new("foo<10"), Version::new("9.0.0")));
        assert!(satisfies_ver(
            &Depend::new("foo<=10"),
            Version::new("9.0.0")
        ));
        assert!(satisfies_ver(&Depend::new("foo>=9"), Version::new("9.0.0")));
        assert!(satisfies_ver(
            &Depend::new("foo=9.0.0"),
            Version::new("9.0.0")
        ));

        assert!(!satisfies_ver(
            &Depend::new("foo>=10"),
            Version::new("9.0.0")
        ));
        assert!(!satisfies_ver(
            &Depend::new("foo<=8"),
            Version::new("9.0.0")
        ));
        assert!(!satisfies_ver(&Depend::new("foo=8"), Version::new("9.0.0")));

        assert!(satisfies_ver(&Depend::new("foo"), Version::new("1")));
        assert!(satisfies_ver(&Depend::new("foo"), Version::new("1.0.0")));
    }
}
