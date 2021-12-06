use crate::{Alpm, Error, LoadedPackage, Package};

use alpm_sys::*;

use std::fmt;

pub unsafe trait IntoPkgAdd: fmt::Debug {
    #[doc(hidden)]
    unsafe fn as_alpm_pkg_t(&self) -> *mut alpm_pkg_t;
    #[doc(hidden)]
    unsafe fn added(self);
}

unsafe impl<'a> IntoPkgAdd for Package<'a> {
    unsafe fn as_alpm_pkg_t(&self) -> *mut alpm_pkg_t {
        self.pkg.as_ptr()
    }
    unsafe fn added(self) {}
}
unsafe impl<'a> IntoPkgAdd for LoadedPackage<'a> {
    unsafe fn as_alpm_pkg_t(&self) -> *mut alpm_pkg_t {
        self.pkg.as_ptr()
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
            Err(err) => Err(AddError { err, pkg }),
        }
    }
}

#[derive(Debug)]
pub struct AddError<P> {
    pub err: Error,
    pub pkg: P,
}

impl<P> fmt::Display for AddError<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.err, f)
    }
}

impl<P: IntoPkgAdd> std::error::Error for AddError<P> {}

impl<P> From<AddError<P>> for Error {
    fn from(err: AddError<P>) -> Error {
        err.err
    }
}
