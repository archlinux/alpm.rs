use crate::{Alpm, AlpmListMut, Result, WithAlpmList};

use alpm_sys::*;

use std::ptr;

impl Alpm {
    pub fn fetch_pkgurl<L: WithAlpmList<String>>(&self, urls: L) -> Result<AlpmListMut<String>> {
        urls.with_alpm_list(|url| {
            let mut out = ptr::null_mut();
            let ret = unsafe { alpm_fetch_pkgurl(self.as_ptr(), url.as_ptr(), &mut out) };
            self.check_ret(ret)?;
            let fetched = unsafe { AlpmListMut::from_ptr(out) };
            Ok(fetched)
        })
    }
}
