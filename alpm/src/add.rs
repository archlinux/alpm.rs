use crate::{Alpm, Error, LoadedPackage, Package};

use alpm_sys::*;

use std::fmt;

#[doc(hidden)]
pub unsafe trait IntoPkgAdd: fmt::Debug {
    unsafe fn as_alpm_pkg_t(&self) -> *mut alpm_pkg_t;
    unsafe fn added(self);
}

unsafe impl IntoPkgAdd for &Package {
    unsafe fn as_alpm_pkg_t(&self) -> *mut alpm_pkg_t {
        self.as_ptr()
    }
    unsafe fn added(self) {}
}

unsafe impl IntoPkgAdd for LoadedPackage<'_> {
    unsafe fn as_alpm_pkg_t(&self) -> *mut alpm_pkg_t {
        self.as_ptr()
    }
    unsafe fn added(self) {
        std::mem::forget(self);
    }
}

impl Alpm {
    pub fn trans_add_pkg<P: IntoPkgAdd>(&self, pkg: P) -> Result<(), AddError<P>> {
        let ret = unsafe { alpm_add_pkg(self.as_ptr(), pkg.as_alpm_pkg_t()) };
        let ok = self.check_ret(ret);
        match ok {
            Ok(_) => {
                unsafe { pkg.added() };
                Ok(())
            }
            Err(err) => Err(AddError { error: err, pkg }),
        }
    }
}

#[derive(Debug)]
pub struct AddError<P> {
    pub error: Error,
    pub pkg: P,
}

impl<P> fmt::Display for AddError<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.error, f)
    }
}

impl<P: IntoPkgAdd> std::error::Error for AddError<P> {}

impl<P> From<AddError<P>> for Error {
    fn from(err: AddError<P>) -> Error {
        err.error
    }
}
