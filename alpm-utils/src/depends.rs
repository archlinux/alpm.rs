use alpm::{DepMod, Depend, Ver, Version};

use std::cmp::Ordering;

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

    if provide.depmod() == DepMod::Any && dep.depmod() != DepMod::Any {
        return false;
    }

    satisfies_ver(dep, provide.version())
}

fn satisfies_ver<'a, V: AsRef<Ver>>(dep: &Depend<'a>, version: V) -> bool {
    let version = version.as_ref();

    if dep.depmod() == DepMod::Any {
        return true;
    }

    let cmp = version.cmp(dep.version());

    match dep.depmod() {
        DepMod::Eq => cmp == Ordering::Equal,
        DepMod::Ge => cmp == Ordering::Greater || cmp == Ordering::Equal,
        DepMod::Le => cmp == Ordering::Less || cmp == Ordering::Equal,
        DepMod::Gt => cmp == Ordering::Greater,
        DepMod::Lt => cmp == Ordering::Less,
        DepMod::Any => true,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_satisfies_ver() {
        assert!(satisfies_ver(&Depend::new("foo>0"), Version::new("9.0.0")));
        assert!(satisfies_ver(&Depend::new("foo<10"), Version::new("9.0.0")));
        assert!(satisfies_ver(&Depend::new("foo<=10"), Version::new("9.0.0")));
        assert!(satisfies_ver(&Depend::new("foo>=9"), Version::new("9.0.0")));
        assert!(satisfies_ver(&Depend::new("foo=9.0.0"), Version::new("9.0.0")));

        assert!(!satisfies_ver(&Depend::new("foo>=10"), Version::new("9.0.0")));
        assert!(!satisfies_ver(&Depend::new("foo<=8"), Version::new("9.0.0")));
        assert!(!satisfies_ver(&Depend::new("foo=8"), Version::new("9.0.0")));

        assert!(satisfies_ver(&Depend::new("foo"), Version::new("1")));
        assert!(satisfies_ver(&Depend::new("foo"), Version::new("1.0.0")));
    }
}
