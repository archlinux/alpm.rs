use alpm_sys::*;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

pub unsafe fn from_cstr<'a>(s: *const c_char) -> &'a str {
    if s.is_null() {
        ""
    } else {
        let s = CStr::from_ptr(s);
        s.to_str().unwrap()
    }
}

pub unsafe fn from_cstr_optional<'a>(s: *const c_char) -> Option<&'a str> {
    s.as_ref()
        .map(|s| CStr::from_ptr(s).to_str().unwrap())
        .filter(|s| !s.is_empty())
}

pub fn to_strlist<S: Into<String>, I: IntoIterator<Item = S>>(list: I) -> *mut alpm_list_t {
    let mut alpmlist = ptr::null_mut();

    for s in list {
        let cs = CString::new(s.into()).unwrap();
        unsafe { alpm_list_append_strdup(&mut alpmlist, cs.as_ptr()) };
    }

    alpmlist
}
