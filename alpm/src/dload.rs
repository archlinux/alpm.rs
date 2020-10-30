#[cfg(not(feature = "git"))]
use crate::{free, utils::*};
use crate::{Alpm, Result};

use alpm_sys::*;

#[cfg(feature = "git")]
use crate::{AlpmListMut, AsRawAlpmList};
#[cfg(feature = "git")]
use std::ptr;

#[cfg(not(feature = "git"))]
use std::ffi::{c_void, CString};

impl Alpm {
    #[cfg(not(feature = "git"))]
    pub fn fetch_pkgurl<S: Into<Vec<u8>>>(&self, url: S) -> Result<String> {
        let url = CString::new(url).unwrap();
        let path = unsafe { alpm_fetch_pkgurl(self.handle, url.as_ptr()) };
        self.check_null(path)?;
        let path_str = unsafe { from_cstr(path) };
        let path_string = path_str.to_string();
        unsafe { free(path as *mut c_void) };
        Ok(path_string)
    }

    #[cfg(feature = "git")]
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
