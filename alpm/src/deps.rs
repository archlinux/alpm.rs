use crate::utils::*;
use crate::{free, Alpm, AlpmList, AlpmListMut, AsRawAlpmList, Db, Package, Ver};

use alpm_sys::alpm_depmod_t::*;
use alpm_sys::*;

use std::ffi::{c_void, CString};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::transmute;

#[derive(Debug)]
pub struct Dep<'a> {
    pub(crate) inner: *mut alpm_depend_t,
    pub(crate) phantom: PhantomData<&'a ()>,
}

unsafe impl<'a> Send for Dep<'a> {}
unsafe impl<'a> Sync for Dep<'a> {}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Depend {
    dep: Dep<'static>,
}

impl Clone for Depend {
    fn clone(&self) -> Self {
        Depend::new(self.to_string())
    }
}

impl fmt::Display for Depend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.dep.fmt(f)
    }
}

impl std::ops::Deref for Depend {
    type Target = Dep<'static>;

    fn deref(&self) -> &Self::Target {
        &self.dep
    }
}

pub trait AsDep {
    fn as_dep(&self) -> &Dep;
}

impl AsDep for Depend {
    fn as_dep(&self) -> &Dep {
        &self.dep
    }
}

impl<'a> AsDep for Dep<'a> {
    fn as_dep(&self) -> &Dep {
        self
    }
}

impl<'a> AsDep for &Dep<'a> {
    fn as_dep(&self) -> &Dep {
        self
    }
}

impl Drop for Depend {
    fn drop(&mut self) {
        unsafe { alpm_dep_free(self.dep.inner) }
    }
}

impl<'a> Hash for Dep<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state);
        self.depmod().hash(state);
        self.version().hash(state);
    }
}

impl<'a> PartialEq for Dep<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
            && self.depmod() == other.depmod()
            && self.version() == other.version()
            && self.desc() == other.desc()
    }
}

impl<'a> Eq for Dep<'a> {}

impl<'a> fmt::Display for Dep<'a> {
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

impl<'a> Into<Vec<u8>> for Dep<'a> {
    fn into(self) -> Vec<u8> {
        unsafe {
            let cs = alpm_dep_compute_string(self.inner);
            let s = std::ffi::CStr::from_ptr(cs);
            let s = s.to_bytes().to_vec();
            free(cs as *mut c_void);
            s
        }
    }
}

impl Depend {
    pub fn new<S: Into<Vec<u8>>>(s: S) -> Depend {
        let s = CString::new(s).unwrap();
        let dep = unsafe { alpm_dep_from_string(s.as_ptr()) };

        Depend {
            dep: Dep {
                inner: dep,
                phantom: PhantomData,
            },
        }
    }

    pub(crate) unsafe fn from_ptr(ptr: *mut alpm_depend_t) -> Depend {
        Depend {
            dep: Dep {
                inner: ptr,
                phantom: PhantomData,
            },
        }
    }
}

impl<'a> Dep<'a> {
    pub fn to_depend(&self) -> Depend {
        Depend::new(self.to_string())
    }

    pub fn name(&self) -> &'a str {
        unsafe { from_cstr((*self.inner).name) }
    }

    pub fn version(&self) -> Option<&'a Ver> {
        unsafe { (*self.inner).version.as_ref().map(|p| Ver::from_ptr(p)) }
    }

    unsafe fn version_unchecked(&self) -> &'a Ver {
        Ver::from_ptr((*self.inner).version)
    }

    pub fn desc(&self) -> &'a str {
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
    pub(crate) unsafe fn from_ptr(ptr: *mut alpm_depend_t) -> Dep<'a> {
        Dep {
            inner: ptr,
            phantom: PhantomData,
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

unsafe impl<'a> Send for DepMissing<'a> {}
unsafe impl<'a> Sync for DepMissing<'a> {}

#[derive(Debug)]
pub struct DepMissing<'a> {
    pub(crate) inner: *mut alpm_depmissing_t,
    pub(crate) phantom: PhantomData<&'a ()>,
}

impl std::ops::Deref for DependMissing {
    type Target = DepMissing<'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug)]
pub struct DependMissing {
    pub(crate) inner: DepMissing<'static>,
}

impl Drop for DependMissing {
    fn drop(&mut self) {
        unsafe { alpm_depmissing_free(self.inner.inner) }
    }
}

impl<'a> DepMissing<'a> {
    pub fn target(&self) -> &str {
        let target = unsafe { (*self.inner).target };
        unsafe { from_cstr(target) }
    }

    pub fn depend(&self) -> Dep {
        let depend = unsafe { (*self.inner).depend };

        unsafe { Dep::from_ptr(depend) }
    }

    pub fn causing_pkg(&self) -> Option<&str> {
        let causing_pkg = unsafe { (*self.inner).causingpkg };
        if causing_pkg.is_null() {
            None
        } else {
            unsafe { Some(from_cstr(causing_pkg)) }
        }
    }
}

impl<'a> AlpmList<'a, Db<'a>> {
    pub fn find_satisfier<S: Into<Vec<u8>>>(&self, dep: S) -> Option<Package<'a>> {
        let dep = CString::new(dep).unwrap();

        let pkg = unsafe { alpm_find_dbs_satisfier(self.handle.handle, self.list, dep.as_ptr()) };
        self.handle.check_null(pkg).ok()?;
        unsafe { Some(Package::new(self.handle, pkg)) }
    }
}

impl<'a> AlpmList<'a, Package<'a>> {
    pub fn find_satisfier<S: Into<Vec<u8>>>(&self, dep: S) -> Option<Package<'a>> {
        let dep = CString::new(dep).unwrap();

        let pkg = unsafe { alpm_find_satisfier(self.list, dep.as_ptr()) };
        self.handle.check_null(pkg).ok()?;
        unsafe { Some(Package::new(self.handle, pkg)) }
    }
}

impl Alpm {
    pub fn check_deps<'a>(
        &self,
        pkgs: impl AsRawAlpmList<'a, Package<'a>>,
        rem: impl AsRawAlpmList<'a, Package<'a>>,
        upgrade: impl AsRawAlpmList<'a, Package<'a>>,
        reverse_deps: bool,
    ) -> AlpmListMut<DependMissing> {
        let reverse_deps = if reverse_deps { 1 } else { 0 };

        let pkgs = unsafe { pkgs.as_raw_alpm_list() };
        let rem = unsafe { rem.as_raw_alpm_list() };
        let upgrade = unsafe { upgrade.as_raw_alpm_list() };

        let ret = unsafe {
            alpm_checkdeps(
                self.handle,
                pkgs.list(),
                rem.list(),
                upgrade.list(),
                reverse_deps,
            )
        };
        AlpmListMut::from_parts(self, ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SigLevel;

    #[test]
    fn test_depend() {
        let dep = Depend::new("abc");
        assert_eq!(dep.name(), "abc");
    }

    #[test]
    fn test_depend_lifetime() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let depends = pkg.depends();
        let vec = depends.iter().collect::<Vec<_>>();
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

        let pkgs1 = handle.localdb().pkgs();
        let pkgs = pkgs1.iter().collect::<Vec<_>>();
        drop(pkgs1);
        let rem = handle.localdb().pkg("ncurses").unwrap();
        let missing = handle.check_deps(
            pkgs.iter(),
            vec![rem].iter(),
            &AlpmListMut::new(&handle),
            true,
        );
        assert_eq!(missing.len(), 9);
    }

    #[test]
    fn test_find_satisfier() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        handle.register_syncdb("core", SigLevel::NONE).unwrap();
        handle.register_syncdb("extra", SigLevel::NONE).unwrap();
        handle.register_syncdb("community", SigLevel::NONE).unwrap();

        let pkg = handle.localdb().pkgs().find_satisfier("linux>0").unwrap();
        assert_eq!(pkg.name(), "linux");

        let pkg = handle.syncdbs().find_satisfier("linux>0").unwrap();
        assert_eq!(pkg.name(), "linux");
    }
}
