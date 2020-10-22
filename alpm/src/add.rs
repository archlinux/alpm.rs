use crate::{Alpm, Error, LoadedPackage, Package};

use alpm_sys::*;

pub unsafe trait IntoPkgAdd {
    unsafe fn as_alpm_pkg_t(&self) -> *mut alpm_pkg_t;
    unsafe fn added(self);
}

unsafe impl<'a> IntoPkgAdd for Package<'a> {
    unsafe fn as_alpm_pkg_t(&self) -> *mut alpm_pkg_t {
        self.pkg
    }
    unsafe fn added(self) {}
}
unsafe impl<'a> IntoPkgAdd for LoadedPackage<'a> {
    unsafe fn as_alpm_pkg_t(&self) -> *mut alpm_pkg_t {
        self.pkg.pkg
    }
    unsafe fn added(self) {
        std::mem::forget(self);
    }
}

impl Alpm {
    pub fn trans_add_pkg<P: IntoPkgAdd>(&self, pkg: P) -> std::result::Result<(), AddError<P>> {
        let ret = unsafe { alpm_add_pkg(self.handle, pkg.as_alpm_pkg_t()) };
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

impl<P> Into<Error> for AddError<P> {
    fn into(self) -> Error {
        self.err
    }
}
