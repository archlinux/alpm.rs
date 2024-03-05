use crate::{Alpm, AlpmListMut, AsAlpmList, Result};

use alpm_sys::*;

use std::ptr;

impl Alpm {
    pub fn fetch_pkgurl<'a, L: AsAlpmList<&'a str>>(&self, urls: L) -> Result<AlpmListMut<String>> {
        urls.with(|url| {
            let mut out = ptr::null_mut();
            let ret = unsafe { alpm_fetch_pkgurl(self.as_ptr(), url.as_ptr(), &mut out) };
            self.check_ret(ret)?;
            let fetched = unsafe { AlpmListMut::from_ptr(out) };
            Ok(fetched)
        })
    }
}
