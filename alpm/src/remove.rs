use crate::{Alpm, Package, Result};

use alpm_sys::*;

impl Alpm {
    pub fn trans_remove_pkg(&self, pkg: &Package) -> Result<()> {
        let ret = unsafe { alpm_remove_pkg(self.as_ptr(), pkg.as_ptr()) };
        self.check_ret(ret)
    }
}
