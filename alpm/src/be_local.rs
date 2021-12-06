use crate::{Package, PackageReason, Result};

use alpm_sys::*;

use std::mem::transmute;

impl<'a> Package<'a> {
    pub fn set_reason(&mut self, reason: PackageReason) -> Result<()> {
        let reason = unsafe { transmute::<PackageReason, _alpm_pkgreason_t>(reason) };
        let ret = unsafe { alpm_pkg_set_reason(self.pkg.as_ptr(), reason) };
        self.handle.check_ret(ret)
    }
}
