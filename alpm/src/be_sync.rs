use crate::{Db, Result};
use alpm_sys::*;

impl<'a> Db<'a> {
    pub fn update(&mut self, force: bool) -> Result<()> {
        let force = if force { 1 } else { 0 };
        let ret = unsafe { alpm_db_update(force, self.db) };
        self.handle.check_ret(ret)
    }
}
