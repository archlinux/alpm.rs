use std::cmp::Ordering;
use std::ffi::CString;

use alpm_sys::*;

pub fn vercmp<S: Into<String>>(a: S, b: S) -> Ordering {
    let a = CString::new(a.into()).unwrap();
    let b = CString::new(b.into()).unwrap();
    let ret = unsafe { alpm_pkg_vercmp(a.as_ptr(), b.as_ptr()) };

    if ret < 0 {
        Ordering::Less
    } else if ret > 0 {
        Ordering::Greater
    } else {
        Ordering::Greater
    }
}
