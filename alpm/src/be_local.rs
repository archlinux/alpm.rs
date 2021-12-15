use crate::{Package, PackageReason, Result};

use alpm_sys::*;

use std::mem::transmute;

impl Package {
    pub fn set_reason(&self, reason: PackageReason) -> Result<()> {
        let reason = unsafe { transmute::<PackageReason, _alpm_pkgreason_t>(reason) };
        let ret = unsafe { alpm_pkg_set_reason(self.as_ptr(), reason) };
        self.check_ret(ret)
    }
}
