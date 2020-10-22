use crate::{PackageReason, Pkg, Result};

use alpm_sys::*;

use std::mem::transmute;

impl<'a> Pkg<'a> {
    pub fn set_reason(&mut self, reason: PackageReason) -> Result<()> {
        let reason = unsafe { transmute::<PackageReason, _alpm_pkgreason_t>(reason) };
        let ret = unsafe { alpm_pkg_set_reason(self.pkg, reason) };
        self.handle.check_ret(ret)
    }
}
