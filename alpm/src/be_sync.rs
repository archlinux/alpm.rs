use crate::Result;
use alpm_sys::*;

use crate::DbMut;

#[cfg(feature = "git")]
use crate::AlpmList;

#[cfg(not(feature = "git"))]
impl<'a> DbMut<'a> {
    pub fn update(&mut self, force: bool) -> Result<bool> {
        let force = if force { 1 } else { 0 };
        let ret = unsafe { alpm_db_update(force, self.db) };
        if ret < 0 {
            Err(self.handle.last_error())
        } else {
            Ok(ret == 1)
        }
    }
}

#[cfg(feature = "git")]
impl<'a> AlpmList<'a, DbMut<'a>> {
    pub fn update(&self, force: bool) -> Result<()> {
        let force = if force { 1 } else { 0 };
        let ret = unsafe { alpm_db_update(self.handle.handle, self.list, force) };
        self.handle.check_ret(ret)
    }
}
