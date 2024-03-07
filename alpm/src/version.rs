use std::cmp::Ordering;
use std::ffi::CStr;
use std::ffi::CString;
use std::fmt;
use std::ops::Deref;
use std::os::raw::c_char;

use alpm_sys::*;

pub fn vercmp<S: Into<Vec<u8>>>(a: S, b: S) -> Ordering {
    let a = Version::new(a);
    let b = Version::new(b);
    a.vercmp(b)
}

#[repr(transparent)]
#[derive(Debug, Eq)]
pub struct Ver(CStr);

impl Ver {
    pub fn new(s: &CStr) -> &Ver {
        unsafe { &*(s as *const CStr as *const Ver) }
    }

    pub fn as_str(&self) -> &str {
        self
    }

    pub fn vercmp<V: AsRef<Ver>>(&self, other: V) -> Ordering {
        unsafe { alpm_pkg_vercmp(self.0.as_ptr(), other.as_ref().0.as_ptr()).cmp(&0) }
    }

    pub(crate) unsafe fn from_ptr<'a>(s: *const c_char) -> &'a Ver {
        Ver::new(CStr::from_ptr(s))
    }
}

impl<'a> From<&'a CStr> for &'a Ver {
    fn from(s: &'a CStr) -> Self {
        Ver::new(s)
    }
}

impl AsRef<Ver> for Ver {
    fn as_ref(&self) -> &Ver {
        self
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

impl PartialOrd<Version> for &Ver {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        self.partial_cmp(&other.as_ver())
    }
}

impl AsRef<str> for Ver {
    fn as_ref(&self) -> &str {
        self
    }
}

impl PartialEq for Ver {
    fn eq(&self, other: &Self) -> bool {
        unsafe { alpm_pkg_vercmp(self.0.as_ptr(), other.0.as_ptr()) == 0 }
    }
}

impl PartialEq<Version> for &Ver {
    fn eq(&self, other: &Version) -> bool {
        unsafe { alpm_pkg_vercmp(self.0.as_ptr(), other.0.as_ptr()) == 0 }
    }
}

#[derive(Debug, Eq, Clone)]
pub struct Version(CString);

impl From<String> for Version {
    fn from(value: String) -> Self {
        Version::new(value.into_bytes())
    }
}

impl Version {
    pub fn new<S: Into<Vec<u8>>>(s: S) -> Self {
        let s = CString::new(s).unwrap();
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
        unsafe { alpm_pkg_vercmp(self.0.as_ptr(), other.0.as_ptr()).partial_cmp(&0) }
    }
}

impl PartialOrd<&Ver> for Version {
    fn partial_cmp(&self, other: &&Ver) -> Option<Ordering> {
        unsafe { alpm_pkg_vercmp(self.0.as_ptr(), other.0.as_ptr()).partial_cmp(&0) }
    }
}

impl AsRef<Ver> for Version {
    fn as_ref(&self) -> &Ver {
        self.as_ver()
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        unsafe { alpm_pkg_vercmp(self.0.as_ptr(), other.0.as_ptr()) == 0 }
    }
}

impl PartialEq<&Ver> for Version {
    fn eq(&self, other: &&Ver) -> bool {
        unsafe { alpm_pkg_vercmp(self.0.as_ptr(), other.0.as_ptr()) == 0 }
    }
}

impl PartialEq<str> for Ver {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<String> for Ver {
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<&str> for Version {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for Version {
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<Ver> for str {
    fn eq(&self, other: &Ver) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<&Ver> for String {
    fn eq(&self, other: &&Ver) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<Version> for &str {
    fn eq(&self, other: &Version) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<Version> for String {
    fn eq(&self, other: &Version) -> bool {
        self == other.as_str()
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
        assert!("2" == Version::new("2"));

        let dep1 = Depend::new("foo=20");
        let dep2 = Depend::new("foo=34");

        assert!(Version::new("1").vercmp(&Version::new("2")) == Ordering::Less);
        assert!(Version::new("2-1").vercmp(&Version::new("2")) == Ordering::Equal);

        assert!(dep1.version() != dep2.version());
        assert!(dep1.version() < dep2.version());
        assert!(Version::new("34") == dep2.version().unwrap());
        assert!(Version::new("34") >= dep2.version().unwrap());
        assert!(dep2.version().unwrap() == Version::new("34"));
        assert!(dep2.version().unwrap() >= Version::new("34"));
        assert!(Version::new("1.9.3-2") < Version::new("1.10.2-1"));
    }
}
