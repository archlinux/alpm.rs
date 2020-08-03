use crate::utils::*;
use crate::{free, Alpm, AlpmList, Db, FreeMethod, Package, Ver};

use alpm_sys::alpm_depmod_t::*;
use alpm_sys::*;

use std::ffi::{c_void, CString};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::transmute;

#[derive(Debug)]
pub struct Depend<'a> {
    pub(crate) inner: *mut alpm_depend_t,
    pub(crate) drop: bool,
    pub(crate) phantom: PhantomData<&'a ()>,
}

impl<'a> Drop for Depend<'a> {
    fn drop(&mut self) {
        if self.drop {
            unsafe { alpm_dep_free(self.inner) }
        }
    }
}

impl<'a> Hash for Depend<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state);
        self.depmod().hash(state);
        self.version().hash(state);
    }
}

impl<'a> PartialEq for Depend<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
            && self.depmod() == other.depmod()
            && self.version() == other.version()
            && self.desc() == other.desc()
    }
}

impl<'a> Eq for Depend<'a> {}

impl<'a> fmt::Display for Depend<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let cs = alpm_dep_compute_string(self.inner);
            let s = from_cstr(cs);
            let err = write!(f, "{}", s);
            free(cs as *mut c_void);
            err
        }
    }
}

impl<S: Into<String>> From<S> for Depend<'static> {
    fn from(s: S) -> Depend<'static> {
        Depend::new(s)
    }
}

impl<'a> Depend<'a> {
    pub fn new<S: Into<String>>(s: S) -> Depend<'static> {
        let s = CString::new(s.into()).unwrap();
        let dep = unsafe { alpm_dep_from_string(s.as_ptr()) };

        Depend {
            inner: dep,
            drop: true,
            phantom: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        unsafe { from_cstr((*self.inner).name) }
    }

    pub fn version(&self) -> Option<&Ver> {
        unsafe { (*self.inner).version.as_ref().map(|p| Ver::from_ptr(p)) }
    }

    unsafe fn version_unchecked(&self) -> &Ver {
        Ver::from_ptr((*self.inner).version)
    }

    pub fn desc(&self) -> &str {
        unsafe { from_cstr((*self.inner).desc) }
    }

    pub fn name_hash(&self) -> u64 {
        unsafe { (*self.inner).name_hash as u64 }
    }

    pub fn depmod(&self) -> DepMod {
        unsafe { transmute::<alpm_depmod_t, DepMod>((*self.inner).mod_) }
    }

    pub fn depmodver(&self) -> DepModVer {
        unsafe {
            match self.depmod() {
                DepMod::Any => DepModVer::Any,
                DepMod::Eq => DepModVer::Eq(self.version_unchecked()),
                DepMod::Ge => DepModVer::Ge(self.version_unchecked()),
                DepMod::Le => DepModVer::Le(self.version_unchecked()),
                DepMod::Gt => DepModVer::Gt(self.version_unchecked()),
                DepMod::Lt => DepModVer::Lt(self.version_unchecked()),
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum DepModVer<'a> {
    Any,
    Eq(&'a Ver),
    Ge(&'a Ver),
    Le(&'a Ver),
    Gt(&'a Ver),
    Lt(&'a Ver),
}

impl From<DepModVer<'_>> for DepMod {
    fn from(d: DepModVer) -> Self {
        match d {
            DepModVer::Any => DepMod::Any,
            DepModVer::Eq(_) => DepMod::Eq,
            DepModVer::Ge(_) => DepMod::Ge,
            DepModVer::Le(_) => DepMod::Le,
            DepModVer::Gt(_) => DepMod::Gt,
            DepModVer::Lt(_) => DepMod::Lt,
        }
    }
}

impl DepModVer<'_> {
    pub fn depmod(self) -> DepMod {
        self.into()
    }
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum DepMod {
    Any = ALPM_DEP_MOD_ANY as u32,
    Eq = ALPM_DEP_MOD_EQ as u32,
    Ge = ALPM_DEP_MOD_GE as u32,
    Le = ALPM_DEP_MOD_LE as u32,
    Gt = ALPM_DEP_MOD_GT as u32,
    Lt = ALPM_DEP_MOD_LT as u32,
}

#[derive(Debug)]
pub struct DepMissing {
    pub(crate) inner: *mut alpm_depmissing_t,
}

impl Drop for DepMissing {
    fn drop(&mut self) {
        unsafe { alpm_depmissing_free(self.inner) }
    }
}

impl DepMissing {
    pub fn target<'a>(&self) -> &'a str {
        let target = unsafe { (*self.inner).target };
        unsafe { from_cstr(target) }
    }

    pub fn depend(&self) -> Depend {
        let depend = unsafe { (*self.inner).depend };

        Depend {
            inner: depend,
            phantom: PhantomData,
            drop: false,
        }
    }

    pub fn causing_pkg<'a>(&self) -> Option<&'a str> {
        let causing_pkg = unsafe { (*self.inner).causingpkg };
        if causing_pkg.is_null() {
            None
        } else {
            unsafe { Some(from_cstr(causing_pkg)) }
        }
    }
}

impl<'a> AlpmList<'a, Db<'a>> {
    pub fn find_satisfier<S: Into<String>>(&self, dep: S) -> Option<Package<'a>> {
        let dep = CString::new(dep.into()).unwrap();

        let pkg = unsafe { alpm_find_dbs_satisfier(self.handle.handle, self.list, dep.as_ptr()) };
        self.handle.check_null(pkg).ok()?;
        unsafe { Some(Package::new(self.handle, pkg)) }
    }
}

impl<'a> AlpmList<'a, Package<'a>> {
    pub fn find_satisfier<S: Into<String>>(&self, dep: S) -> Option<Package<'a>> {
        let dep = CString::new(dep.into()).unwrap();

        let pkg = unsafe { alpm_find_satisfier(self.list, dep.as_ptr()) };
        self.handle.check_null(pkg).ok()?;
        unsafe { Some(Package::new(self.handle, pkg)) }
    }
}

impl Alpm {
    pub fn check_deps<'a>(
        &self,
        pkgs: impl IntoIterator<Item = Package<'a>>,
        rem: impl IntoIterator<Item = Package<'a>>,
        upgrade: impl IntoIterator<Item = Package<'a>>,
        reverse_deps: bool,
    ) -> AlpmList<DepMissing> {
        let reverse_deps = if reverse_deps { 1 } else { 0 };
        let mut pkglist = std::ptr::null_mut();
        let mut remlist = std::ptr::null_mut();
        let mut upgradelist = std::ptr::null_mut();

        for pkg in pkgs {
            pkglist = unsafe { alpm_list_add(pkglist, pkg.pkg as *mut c_void) };
        }

        for pkg in rem {
            remlist = unsafe { alpm_list_add(remlist, pkg.pkg as *mut c_void) };
        }

        for pkg in upgrade {
            upgradelist = unsafe { alpm_list_add(upgradelist, pkg.pkg as *mut c_void) };
        }

        let ret =
            unsafe { alpm_checkdeps(self.handle, pkglist, remlist, upgradelist, reverse_deps) };
        unsafe { alpm_list_free(pkglist) };
        unsafe { alpm_list_free(remlist) };
        unsafe { alpm_list_free(upgradelist) };
        AlpmList::new(self, ret, FreeMethod::FreeDepMissing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SigLevel;

    #[test]
    fn test_depend_lifetime() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let depends = pkg.depends();
        let vec = depends.collect::<Vec<_>>();
        drop(pkg);
        drop(db);
        println!("{:?}", vec);
    }

    #[test]
    fn test_eq() {
        assert_eq!(Depend::new("foo=1"), Depend::new("foo=1"));
        assert!(Depend::new("foo=2") != Depend::new("foo=1"));
    }

    #[test]
    fn test_check_deps() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        handle.register_syncdb("extra", SigLevel::NONE).unwrap();
        handle.register_syncdb("community", SigLevel::NONE).unwrap();

        let pkgs = handle.localdb().pkgs().unwrap().collect::<Vec<_>>();
        let rem = handle.localdb().pkg("ncurses").unwrap();
        let missing = handle.check_deps(pkgs, vec![rem], vec![], true);
        assert_eq!(missing.len(), 9);
    }

    #[test]
    fn test_find_satisfier() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        handle.register_syncdb("core", SigLevel::NONE).unwrap();
        handle.register_syncdb("extra", SigLevel::NONE).unwrap();
        handle.register_syncdb("community", SigLevel::NONE).unwrap();

        let pkg = handle
            .localdb()
            .pkgs()
            .unwrap()
            .find_satisfier("linux>0")
            .unwrap();
        assert_eq!(pkg.name(), "linux");

        let pkg = handle.syncdbs().find_satisfier("linux>0").unwrap();
        assert_eq!(pkg.name(), "linux");
    }
}
