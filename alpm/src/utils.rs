use std::ffi::CStr;
use std::os::raw::c_char;

pub unsafe fn from_cstr<'a>(s: *const c_char) -> &'a str {
    debug_assert!(!s.is_null(), "str is null");
    unsafe { CStr::from_ptr(s).to_str().unwrap() }
}

pub unsafe fn from_cstr_optional<'a>(s: *const c_char) -> Option<&'a str> {
    unsafe { s.as_ref().map(|s| CStr::from_ptr(s).to_str().unwrap()) }
}
