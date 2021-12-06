use crate::Result;
use alpm_sys::*;

use crate::{AlpmList, DbMut};

impl<'a> AlpmList<'a, DbMut<'a>> {
    pub fn update(&self, force: bool) -> Result<bool> {
        let force = if force { 1 } else { 0 };
        let ret = unsafe { alpm_db_update(self.handle.as_ptr(), self.list, force) };
        if ret == -1 {
            Err(self.handle.last_error())
        } else {
            Ok(ret == 1)
        }
    }
}
