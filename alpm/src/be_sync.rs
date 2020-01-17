use crate::{DbMut, Result};
use alpm_sys::*;

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
