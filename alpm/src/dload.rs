use crate::{Alpm, AlpmListMut, IntoRawAlpmList, Result};

use alpm_sys::*;

use std::ptr;

impl Alpm {
    pub fn fetch_pkgurl<'a, L: IntoRawAlpmList<'a, String>>(
        &'a self,
        urls: L,
    ) -> Result<AlpmListMut<'a, String>> {
        let mut out = ptr::null_mut();
        let list = unsafe { urls.into_raw_alpm_list() };
        let ret = unsafe { alpm_fetch_pkgurl(self.as_ptr(), list.list(), &mut out) };
        self.check_ret(ret)?;
        let fetched = unsafe { AlpmListMut::from_ptr(out) };
        Ok(fetched)
    }
}
