use crate::utils::*;
use crate::{free, Alpm, Result};

use alpm_sys::*;

use std::ffi::{c_void, CString};

impl Alpm {
    pub fn fetch_pkgurl<S: Into<String>>(&self, url: S) -> Result<String> {
        let url = CString::new(url.into())?;
        let path = unsafe { alpm_fetch_pkgurl(self.handle, url.as_ptr()) };
        self.check_null(path)?;
        let path_str = unsafe { from_cstr(path) };
        let path_string = path_str.to_string();
        unsafe { free(path as *mut c_void) };
        Ok(path_string)
    }
}
