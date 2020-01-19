use crate::{Alpm, Package, Result};

use alpm_sys::*;

impl Alpm {
    pub fn trans_add_pkg(&self, pkg: Package) -> Result<()> {
        let ret = unsafe { alpm_add_pkg(self.handle, pkg.pkg) };
        self.check_ret(ret)
    }
}
