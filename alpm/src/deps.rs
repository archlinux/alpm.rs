use crate::utils::*;
use crate::{free, Alpm, AlpmList, Db, FreeMethod, Package, Result};

use alpm_sys::alpm_depmod_t::*;
use alpm_sys::*;

use std::ffi::{c_void, CString};
use std::fmt;
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

impl<'a> Depend<'a> {
    pub fn new<S: Into<String>>(s: S) -> Result<Depend<'static>> {
        let s = CString::new(s.into())?;
        let dep = unsafe { alpm_dep_from_string(s.as_ptr()) };

        let dep = Depend {
            inner: dep,
            drop: true,
            phantom: PhantomData,
        };

        Ok(dep)
    }

    pub fn name(&self) -> &str {
        unsafe { from_cstr((*self.inner).name) }
    }

    pub fn version(&self) -> &str {
        unsafe { from_cstr((*self.inner).version) }
    }

    pub fn desc(&self) -> &str {
        unsafe { from_cstr((*self.inner).desc) }
    }

    pub fn name_hash(&self) -> u64 {
        unsafe { (*self.inner).name_hash as u64 }
    }

    pub fn depmod(&self) -> Depmod {
        unsafe { transmute::<alpm_depmod_t, Depmod>((*self.inner).mod_) }
    }
}

#[repr(u32)]
#[derive(Debug)]
pub enum Depmod {
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

    pub fn causing_pkg<'a>(&self) -> Result<Option<&'a str>> {
        let causing_pkg = unsafe { (*self.inner).causingpkg };
        if causing_pkg.is_null() {
            return Ok(None);
        }

        let ret = unsafe { from_cstr(causing_pkg) };
        Ok(Some(ret))
    }
}

impl Alpm {
    pub fn find_dbs_satisfier<S: Into<String>>(
        &self,
        dbs: AlpmList<Db>,
        dep: S,
    ) -> Result<Package> {
        let dep = CString::new(dep.into()).unwrap();

        let pkg = unsafe { alpm_find_dbs_satisfier(self.handle, dbs.item, dep.as_ptr()) };
        self.check_null(pkg)?;

        let pkg = Package {
            handle: self,
            pkg,
            drop: false,
        };

        Ok(pkg)
    }

    pub fn find_satisfier<S: Into<String>>(
        &self,
        pkgs: AlpmList<Package>,
        dep: S,
    ) -> Result<Package> {
        let dep = CString::new(dep.into()).unwrap();

        let pkg = unsafe { alpm_find_satisfier(pkgs.item, dep.as_ptr()) };
        self.check_null(pkg)?;

        let pkg = Package {
            handle: self,
            pkg,
            drop: false,
        };

        Ok(pkg)
    }

    pub fn check_deps(
        &self,
        pkgs: AlpmList<Package>,
        rem: AlpmList<Package>,
        upgrade: AlpmList<Package>,
        reverse_deps: bool,
    ) -> AlpmList<DepMissing> {
        let reverse_deps = if reverse_deps { 1 } else { 0 };
        let list =
            unsafe { alpm_checkdeps(self.handle, pkgs.item, rem.item, upgrade.item, reverse_deps) };

        AlpmList {
            handle: self,
            item: list,
            free: FreeMethod::FreeDepMissing,
            _marker: PhantomData,
        }
    }
}
