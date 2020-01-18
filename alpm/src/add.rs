use crate::{Alpm, LoadedPackage, Package, Result};

use alpm_sys::*;

pub unsafe trait PkgAdd {
    fn as_alpm_pkg_t(&self) -> *mut alpm_pkg_t;
}

unsafe impl<'a> PkgAdd for Package<'a> {
    fn as_alpm_pkg_t(&self) -> *mut alpm_pkg_t {
        self.pkg
    }
}
unsafe impl<'a> PkgAdd for LoadedPackage<'a> {
    fn as_alpm_pkg_t(&self) -> *mut alpm_pkg_t {
        self.pkg.pkg
    }
}

impl Alpm {
    pub fn trans_add_pkg(&self, pkg: Package) -> Result<()> {
        let ret = unsafe { alpm_add_pkg(self.handle, pkg.pkg) };
        let ok = self.check_ret(ret);
        if ok.is_ok() {
            std::mem::forget(pkg);
        }
        ok
    }
}
