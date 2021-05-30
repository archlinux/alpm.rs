use std::ffi::CStr;
use std::os::raw::c_char;

pub unsafe fn from_cstr<'a>(s: *const c_char) -> &'a str {
    CStr::from_ptr(s).to_str().unwrap()
}

pub unsafe fn from_cstr_optional<'a>(s: *const c_char) -> Option<&'a str> {
    s.as_ref().map(|s| CStr::from_ptr(s).to_str().unwrap())
}
