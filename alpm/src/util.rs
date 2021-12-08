use std::ffi::{CStr, CString};
use std::fmt;

use alpm_sys::*;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct ChecksumError;

impl fmt::Display for ChecksumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to compute checksum")
    }
}

impl std::error::Error for ChecksumError {}

pub fn compute_md5sum<S: Into<Vec<u8>>>(s: S) -> Result<String, ChecksumError> {
    let s = CString::new(s).unwrap();
    let ret = unsafe { alpm_compute_md5sum(s.as_ptr()) };
    if ret.is_null() {
        return Err(ChecksumError);
    }

    let s = unsafe { CStr::from_ptr(ret).to_str().unwrap() };
    Ok(s.into())
}

pub fn compute_sha256sum<S: Into<Vec<u8>>>(s: S) -> Result<String, ChecksumError> {
    let s = CString::new(s).unwrap();
    let ret = unsafe { alpm_compute_sha256sum(s.as_ptr()) };
    if ret.is_null() {
        return Err(ChecksumError);
    }

    let s = unsafe { CStr::from_ptr(ret).to_str().unwrap() };
    Ok(s.into())
}
