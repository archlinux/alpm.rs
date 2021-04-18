use crate::{Alpm, AlpmListMut, AsRawAlpmList, Result};

use alpm_sys::*;

use std::ptr;

impl Alpm {
    pub fn fetch_pkgurl<'a, L: AsRawAlpmList<'a, String>>(
        &'a self,
        urls: L,
    ) -> Result<AlpmListMut<'a, String>> {
        let mut out = ptr::null_mut();
        let list = unsafe { urls.as_raw_alpm_list() };
        let ret = unsafe { alpm_fetch_pkgurl(self.handle, list.list(), &mut out) };
        self.check_ret(ret)?;
        let fetched = AlpmListMut::from_parts(self, out);
        Ok(fetched)
    }
}
