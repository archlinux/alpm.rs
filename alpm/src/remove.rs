use std::ffi::CString;

use crate::{Alpm, Package, Result};

use alpm_sys::*;

impl Alpm {
    pub fn trans_remove_pkg(&self, pkg: &Package) -> Result<()> {
        let ret = unsafe { alpm_remove_pkg(self.as_ptr(), pkg.as_ptr()) };
        self.check_ret(ret)
    }

    /// Remove a package from the transaction by name.
    ///
    /// This is a convenience method that looks up the package in the local database
    /// and adds it to the removal transaction. It avoids borrow checker conflicts
    /// that occur when holding `&Package` references across transaction operations
    /// like `trans_prepare()`.
    ///
    /// # Example
    /// ```no_run
    /// use alpm::{Alpm, TransFlag};
    ///
    /// let mut handle = Alpm::new("/", "/var/lib/pacman").unwrap();
    /// let names = ["package1", "package2"];
    ///
    /// handle.trans_init(TransFlag::empty()).unwrap();
    /// for name in names {
    ///     handle.trans_remove_pkg_by_name(name).unwrap();
    /// }
    /// handle.trans_prepare().unwrap();
    /// handle.trans_commit().unwrap();
    /// handle.trans_release().unwrap();
    /// ```
    pub fn trans_remove_pkg_by_name<S: Into<Vec<u8>>>(&self, name: S) -> Result<()> {
        let name = CString::new(name).unwrap();
        let pkg = unsafe { alpm_db_get_pkg(alpm_get_localdb(self.as_ptr()), name.as_ptr()) };
        self.check_null(pkg)?;
        let ret = unsafe { alpm_remove_pkg(self.as_ptr(), pkg) };
        self.check_ret(ret)
    }
}
