use crate::{Package, Result, Trans};

use alpm_sys::*;

impl<'a> Trans<'a> {
    pub fn add_pkg(&mut self, pkg: &Package) -> Result<()> {
        let ret = unsafe { alpm_add_pkg(self.handle.handle, pkg.pkg) };
        self.handle.check_ret(ret)
    }
}
