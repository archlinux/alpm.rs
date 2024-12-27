use crate::utils::*;
use crate::{free, Alpm, AlpmList, AlpmListMut, AsAlpmList, Db, Package, Pkg, Ver};

use alpm_sys::alpm_depmod_t::*;
use alpm_sys::*;

use std::ffi::{c_void, CString};
use std::fmt;
use std::mem::transmute;
use std::ptr::NonNull;

#[repr(transparent)]
pub struct Dep {
    inner: alpm_depend_t,
}

impl fmt::Debug for Dep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dep")
            .field("name", &self.name())
            .field("version", &self.version())
            .field("desc", &self.desc())
            .field("depmod", &self.depmod())
            .field("name_hash", &self.name_hash())
            .finish()
    }
}

unsafe impl Send for Dep {}
unsafe impl Sync for Dep {}

pub struct Depend {
    dep: NonNull<alpm_depend_t>,
}

impl PartialEq for Depend {
    fn eq(&self, other: &Self) -> bool {
        self.as_dep() == other.as_dep()
    }
}

impl AsRef<Dep> for Dep {
    fn as_ref(&self) -> &Dep {
        self
    }
}

impl Clone for Depend {
    fn clone(&self) -> Self {
        let ptr = unsafe { alpm_dep_compute_string(self.as_ptr()) };
        assert!(!ptr.is_null(), "failed to compute string for dep");
        let dep = unsafe { alpm_dep_from_string(ptr) };
        unsafe { free(ptr as _) };
        assert!(!dep.is_null(), "failed to create dep from string");
        unsafe { Depend::from_ptr(dep) }
    }
}

impl fmt::Debug for Depend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_dep(), f)
    }
}

impl fmt::Display for Depend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_dep().fmt(f)
    }
}

impl std::ops::Deref for Depend {
    type Target = Dep;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.dep.as_ptr() as *mut alpm_depend_t as *const alpm_depend_t as *const Dep) }
    }
}

impl Drop for Depend {
    fn drop(&mut self) {
        unsafe { alpm_dep_free(self.dep.as_ptr()) }
    }
}

impl PartialEq for Dep {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
            && self.depmod() == other.depmod()
            && self.version() == other.version()
            && self.desc() == other.desc()
    }
}

impl fmt::Display for Dep {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let cs = alpm_dep_compute_string(self.as_ptr());
            assert!(!cs.is_null(), "failed to compute string for dep");
            let s = from_cstr(cs);
            let err = f.write_str(s);
            free(cs as *mut c_void);
            err
        }
    }
}

impl From<Dep> for Vec<u8> {
    fn from(dep: Dep) -> Vec<u8> {
        unsafe {
            let cs = alpm_dep_compute_string(dep.as_ptr());
            assert!(!cs.is_null(), "failed to compute string for dep");
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
        assert!(!dep.is_null(), "failed to create dep from string");
        unsafe {
            Depend {
                dep: NonNull::new_unchecked(dep),
            }
        }
    }

    pub(crate) unsafe fn from_ptr(ptr: *mut alpm_depend_t) -> Depend {
        Depend {
            dep: NonNull::new_unchecked(ptr),
        }
    }

    pub fn as_dep(&self) -> &Dep {
        self
    }
}

impl AsRef<Dep> for Depend {
    fn as_ref(&self) -> &Dep {
        self.as_dep()
    }
}

impl Dep {
    pub(crate) unsafe fn from_ptr<'a>(ptr: *const alpm_depend_t) -> &'a Dep {
        &*(ptr as *const Dep)
    }

    pub(crate) fn as_ptr(&self) -> *const alpm_depend_t {
        &self.inner
    }

    pub fn dep(&self) -> &Dep {
        unsafe { Dep::from_ptr(self.as_ptr()) }
    }

    pub fn to_depend(&self) -> Depend {
        Depend::new(self.to_string())
    }

    pub fn name(&self) -> &str {
        unsafe { from_cstr((*self.as_ptr()).name) }
    }

    pub fn version(&self) -> Option<&Ver> {
        unsafe { (*self.as_ptr()).version.as_ref().map(|p| Ver::from_ptr(p)) }
    }

    unsafe fn version_unchecked(&self) -> &Ver {
        Ver::from_ptr((*self.as_ptr()).version)
    }

    pub fn desc(&self) -> Option<&str> {
        unsafe { from_cstr_optional((*self.as_ptr()).desc) }
    }

    pub fn name_hash(&self) -> u64 {
        #[allow(clippy::unnecessary_cast)]
        unsafe {
            (*self.as_ptr()).name_hash as u64
        }
    }

    pub fn depmod(&self) -> DepMod {
        unsafe { transmute::<alpm_depmod_t, DepMod>((*self.as_ptr()).mod_) }
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

#[derive(Debug, Eq, PartialEq, Copy, Clone, PartialOrd)]
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

#[repr(transparent)]
pub struct DepMissing {
    inner: alpm_depmissing_t,
}

unsafe impl Sync for DepMissing {}
unsafe impl Send for DepMissing {}

impl fmt::Debug for DepMissing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DepMissing")
            .field("target", &self.target())
            .field("depend", &self.depend())
            .field("causing_pkg", &self.causing_pkg())
            .finish()
    }
}

impl AsRef<DepMissing> for DepMissing {
    fn as_ref(&self) -> &DepMissing {
        self
    }
}

impl std::ops::Deref for DependMissing {
    type Target = DepMissing;

    fn deref(&self) -> &Self::Target {
        unsafe { DepMissing::from_ptr(self.inner.as_ptr()) }
    }
}

pub struct DependMissing {
    inner: NonNull<alpm_depmissing_t>,
}

unsafe impl Sync for DependMissing {}
unsafe impl Send for DependMissing {}

impl fmt::Debug for DependMissing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl AsRef<DependMissing> for DependMissing {
    fn as_ref(&self) -> &DependMissing {
        self
    }
}

impl Drop for DependMissing {
    fn drop(&mut self) {
        unsafe { alpm_depmissing_free(self.inner.as_ptr()) }
    }
}

impl DependMissing {
    pub(crate) unsafe fn from_ptr(ptr: *mut alpm_depmissing_t) -> DependMissing {
        DependMissing {
            inner: NonNull::new_unchecked(ptr),
        }
    }
}

impl DepMissing {
    pub(crate) unsafe fn from_ptr<'a>(ptr: *mut alpm_depmissing_t) -> &'a DepMissing {
        &*(ptr as *mut DepMissing)
    }

    pub(crate) fn as_ptr(&self) -> *const alpm_depmissing_t {
        &self.inner
    }

    pub fn target(&self) -> &str {
        let target = unsafe { (*self.as_ptr()).target };
        unsafe { from_cstr(target) }
    }

    pub fn depend(&self) -> &Dep {
        let depend = unsafe { (*self.as_ptr()).depend };

        unsafe { Dep::from_ptr(depend) }
    }

    pub fn causing_pkg(&self) -> Option<&str> {
        let causing_pkg = unsafe { (*self.as_ptr()).causingpkg };
        if causing_pkg.is_null() {
            None
        } else {
            unsafe { Some(from_cstr(causing_pkg)) }
        }
    }
}

impl<'a> AlpmList<'_, &'a Db> {
    pub fn find_satisfier<S: Into<Vec<u8>>>(&self, dep: S) -> Option<&'a Package> {
        let dep = CString::new(dep).unwrap();
        let handle = self.first().map(|p| p.handle_ptr())?;

        let pkg = unsafe { alpm_find_dbs_satisfier(handle, self.as_ptr(), dep.as_ptr()) };
        if pkg.is_null() {
            None
        } else {
            unsafe { Some(Package::from_ptr(pkg)) }
        }
    }
}

impl<'a> AlpmList<'_, &'a Package> {
    pub fn find_satisfier<S: Into<Vec<u8>>>(&self, dep: S) -> Option<&'a Package> {
        let dep = CString::new(dep).unwrap();

        let pkg = unsafe { alpm_find_satisfier(self.as_ptr(), dep.as_ptr()) };
        if pkg.is_null() {
            None
        } else {
            unsafe { Some(Package::from_ptr(pkg)) }
        }
    }
}

impl Alpm {
    pub fn check_deps<'a>(
        &self,
        pkgs: impl AsAlpmList<&'a Pkg>,
        rem: impl AsAlpmList<&'a Pkg>,
        upgrade: impl AsAlpmList<&'a Pkg>,
        reverse_deps: bool,
    ) -> AlpmListMut<DependMissing> {
        let reverse_deps = if reverse_deps { 1 } else { 0 };

        pkgs.with(|pkgs| {
            rem.with(|rem| {
                upgrade.with(|upgrade| {
                    let ret = unsafe {
                        alpm_checkdeps(
                            self.as_ptr(),
                            pkgs.as_ptr(),
                            rem.as_ptr(),
                            upgrade.as_ptr(),
                            reverse_deps,
                        )
                    };
                    unsafe { AlpmListMut::from_ptr(ret) }
                })
            })
        })
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

        let dep = Depend::new("<3");
        assert_eq!(dep.name(), "");
        assert_eq!(dep.version().unwrap().as_str(), "3");
    }

    #[test]
    fn test_depend_lifetime() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let depends = pkg.depends();
        let vec = depends.iter().collect::<Vec<_>>();
        println!("{:?}", vec);
    }

    #[test]
    fn test_eq() {
        assert_eq!(Depend::new("foo=1"), Depend::new("foo=1"));
        assert_ne!(Depend::new("foo=1-1"), Depend::new("foo=1-2"));
        assert_ne!(Depend::new("foo=2"), Depend::new("foo=1"));
    }

    #[test]
    fn test_check_deps() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        handle.register_syncdb("extra", SigLevel::NONE).unwrap();
        handle.register_syncdb("community", SigLevel::NONE).unwrap();

        let pkgs1 = handle.localdb().pkgs();
        let pkgs = pkgs1.iter().collect::<Vec<_>>();
        let rem = handle.localdb().pkg("ncurses").unwrap();
        let missing = handle.check_deps(
            pkgs.iter(),
            vec![rem].iter(),
            AlpmListMut::<&Pkg>::new(),
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
