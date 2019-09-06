use std::cmp::Ordering;
use std::ffi::CString;
use std::ops::Deref;
use std::fmt;

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
        Ordering::Equal
    }
}


#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Version(str);

impl Version {
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Version {
        unsafe { &*(s.as_ref() as *const str as *const Version) }
    }
}

impl Deref for Version {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<str> for Version {
    fn eq(&self, other: &str) -> bool {
        &self.0 == other
    }
}


impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(vercmp(self.to_string(), other.to_string()))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        vercmp(self.to_string(), other.to_string())
    }
}

impl AsRef<str> for Version {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Version {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(Version::new("0") <= Version::new("1"));
        assert!(Version::new("2") <= Version::new("2"));
        assert!(Version::new("2") > Version::new("1"));
        assert!(Version::new("2") < Version::new("3"));
        assert!(Version::new("2") >= Version::new("1"));
        assert!(Version::new("2") >= Version::new("2"));
        assert!(Version::new("2") == Version::new("2"));
    }
}
