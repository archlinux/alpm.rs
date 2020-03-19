use crate::utils::*;
use crate::{free, Alpm, Result};

use alpm_sys::*;

use std::ffi::{c_void, CString};

impl Alpm {
    pub fn fetch_pkgurl(&self, url: impl AsRef<str>) -> Result<String> {
        let url = CString::new(url.as_ref()).unwrap();
        let path = unsafe { alpm_fetch_pkgurl(self.handle, url.as_ptr()) };
        self.check_null(path)?;
        let path_str = unsafe { from_cstr(path) };
        let path_string = path_str.to_string();
        unsafe { free(path as *mut c_void) };
        Ok(path_string)
    }
}
