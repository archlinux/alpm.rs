use std::ffi::{CStr, CString};

use alpm_sys::*;

pub fn compute_md5sum(s: impl AsRef<str>) -> Result<String, ()> {
    let s = CString::new(s.as_ref()).unwrap();
    let ret = unsafe { alpm_compute_md5sum(s.as_ptr()) };
    if ret.is_null() {
        return Err(());
    }

    let s = unsafe { CStr::from_ptr(ret).to_str().unwrap() };
    Ok(s.into())
}

pub fn compute_sha256sum(s: impl AsRef<str>) -> Result<String, ()> {
    let s = CString::new(s.as_ref()).unwrap();
    let ret = unsafe { alpm_compute_sha256sum(s.as_ptr()) };
    if ret.is_null() {
        return Err(());
    }

    let s = unsafe { CStr::from_ptr(ret).to_str().unwrap() };
    Ok(s.into())
}
