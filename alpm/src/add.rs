use std::ffi::CString;
use std::fmt;

use crate::{Alpm, Error, LoadedPackage, Package, Result};

use alpm_sys::*;

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
    pub fn trans_add_pkg<P: IntoPkgAdd>(&self, pkg: P) -> std::result::Result<(), AddError<P>> {
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

    /// Add a package to the transaction by name, searching all sync databases.
    ///
    /// This is a convenience method that searches all registered sync databases
    /// for a package matching the given name and adds it to the transaction.
    /// It avoids borrow checker conflicts that occur when holding `&Package`
    /// references across transaction operations like `trans_prepare()`.
    ///
    /// # Example
    /// ```no_run
    /// use alpm::{Alpm, SigLevel, TransFlag};
    ///
    /// let mut handle = Alpm::new("/", "/var/lib/pacman").unwrap();
    /// handle.register_syncdb("core", SigLevel::NONE).unwrap();
    /// let names = ["linux", "base"];
    ///
    /// handle.trans_init(TransFlag::empty()).unwrap();
    /// for name in names {
    ///     handle.trans_add_pkg_by_name(name).unwrap();
    /// }
    /// handle.trans_prepare().unwrap();
    /// handle.trans_commit().unwrap();
    /// handle.trans_release().unwrap();
    /// ```
    pub fn trans_add_pkg_by_name<S: Into<Vec<u8>>>(&self, name: S) -> Result<()> {
        let name = CString::new(name).unwrap();
        let dbs = unsafe { alpm_get_syncdbs(self.as_ptr()) };
        let pkg = unsafe { alpm_find_dbs_satisfier(self.as_ptr(), dbs, name.as_ptr()) };
        self.check_null(pkg)?;
        let ret = unsafe { alpm_add_pkg(self.as_ptr(), pkg) };
        self.check_ret(ret)
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
