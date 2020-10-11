use crate::utils::*;
use crate::{free, Alpm, Result};

use alpm_sys::*;

#[cfg(feature = "git")]
use crate::{AlpmList, FreeMethod};
#[cfg(feature = "git")]
use std::ptr;

#[cfg(not(feature = "git"))]
use std::ffi::{c_void, CString};

impl Alpm {
    #[cfg(not(feature = "git"))]
    pub fn fetch_pkgurl<S: Into<String>>(&self, url: S) -> Result<String> {
        let url = CString::new(url.into()).unwrap();
        let path = unsafe { alpm_fetch_pkgurl(self.handle, url.as_ptr()) };
        self.check_null(path)?;
        let path_str = unsafe { from_cstr(path) };
        let path_string = path_str.to_string();
        unsafe { free(path as *mut c_void) };
        Ok(path_string)
    }

    #[cfg(feature = "git")]
    pub fn fetch_pkgurl<'a, S: Into<String>>(
        &'a self,
        urls: impl IntoIterator<Item = S>,
    ) -> Result<AlpmList<'a, String>> {
        let list = to_strlist(urls);
        let mut out = ptr::null_mut();
        let ret = unsafe { alpm_fetch_pkgurl(self.handle, list, &mut out) };
        unsafe { alpm_list_free_inner(list, Some(free)) };
        unsafe { alpm_list_free(list) };
        self.check_ret(ret)?;
        let fetched = AlpmList::new(self, out, FreeMethod::FreeInner);
        Ok(fetched)
    }
}
