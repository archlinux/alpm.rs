use crate::{Error, Result};
use alpm_sys::*;

use crate::{AlpmList, DbMut};

impl<'a> AlpmList<'a, DbMut<'a>> {
    pub fn update(&self, force: bool) -> Result<bool> {
        let first = self.first().ok_or(Error::WrongArgs)?;
        let force = if force { 1 } else { 0 };
        let ret = unsafe { alpm_db_update(first.handle_ptr(), self.as_ptr(), force) };
        if ret == -1 {
            Err(first.last_error())
        } else {
            Ok(ret == 1)
        }
    }
}
