use std::cmp::Ordering;
use std::ffi::CStr;
use std::ffi::CString;
use std::fmt;
use std::ops::Deref;
use std::os::raw::c_char;

use alpm_sys::*;

pub fn vercmp<S: Into<String>>(a: S, b: S) -> Ordering {
    let a = CString::new(a.into()).unwrap();
    let b = CString::new(b.into()).unwrap();
    unsafe { alpm_pkg_vercmp(a.as_ptr(), b.as_ptr()).cmp(&0) }
}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Ver(CStr);

impl Ver {
    pub fn new(s: &CStr) -> &Ver {
        unsafe { &*(s as *const CStr as *const Ver) }
    }

    pub unsafe fn from_ptr<'a>(s: *const c_char) -> &'a Ver {
        if s.is_null() {
            Ver::new(CStr::from_bytes_with_nul_unchecked(&[0]))
        } else {
            Ver::new(CStr::from_ptr(s))
        }
    }
}

impl<'a> From<&'a CStr> for &'a Ver {
    fn from(s: &'a CStr) -> Self {
        Ver::new(s)
    }
}

impl Deref for Ver {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.to_str().unwrap()
    }
}

impl fmt::Display for Ver {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self)
    }
}

impl PartialOrd for Ver {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        unsafe { alpm_pkg_vercmp(self.0.as_ptr(), other.0.as_ptr()).partial_cmp(&0) }
    }
}

impl Ord for Ver {
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { alpm_pkg_vercmp(self.0.as_ptr(), other.0.as_ptr()).cmp(&0) }
    }
}

impl AsRef<str> for Ver {
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsRef<Ver> for Ver {
    fn as_ref(&self) -> &Ver {
        self
    }
}

impl PartialEq<str> for Ver {
    fn eq(&self, other: &str) -> bool {
        self.0.to_str().unwrap() == other
    }
}

impl PartialEq<Version> for &Ver {
    fn eq(&self, other: &Version) -> bool {
        other == self
    }
}

impl PartialOrd<Version> for &Ver {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        self.partial_cmp(&other.as_ver())
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Version(CString);

impl Version {
    pub fn new<S: Into<String>>(s: S) -> Self {
        let s = CString::new(s.into()).unwrap();
        Version(s)
    }

    pub fn as_ver(&self) -> &Ver {
        self
    }
}

impl fmt::Display for Version {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.as_ver())
    }
}

impl Deref for Version {
    type Target = Ver;
    fn deref(&self) -> &Self::Target {
        Ver::new(&self.0)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialEq<&str> for Version {
    fn eq(&self, other: &&str) -> bool {
        self.0.to_str().unwrap() == *other
    }
}

impl PartialEq<&Ver> for Version {
    fn eq(&self, other: &&Ver) -> bool {
        self.0.as_c_str() == &other.0
    }
}

impl PartialOrd<&Ver> for Version {
    fn partial_cmp(&self, other: &&Ver) -> Option<Ordering> {
        self.as_ver().partial_cmp(other)
    }
}

impl AsRef<Ver> for Version {
    fn as_ref(&self) -> &Ver {
        self.as_ver()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Depend;

    #[test]
    fn test_version() {
        assert!(Version::new("0") <= Version::new("1"));
        assert!(Version::new("2") <= Version::new("2"));
        assert!(Version::new("2") > Version::new("1"));
        assert!(Version::new("2") < Version::new("3"));
        assert!(Version::new("2") >= Version::new("1"));
        assert!(Version::new("2") >= Version::new("2"));
        assert!(Version::new("2") == Version::new("2"));
        assert!(Version::new("2") == "2");

        let dep1 = Depend::new("foo=20");
        let dep2 = Depend::new("foo=34");

        assert!(dep1.version() != dep2.version());
        assert!(dep1.version() < dep2.version());
        assert!(Version::new("34") == dep2.version());
        assert!(Version::new("34") >= dep2.version());
        assert!(dep2.version() == Version::new("34"));
        assert!(dep2.version() >= Version::new("34"));
    }
}
