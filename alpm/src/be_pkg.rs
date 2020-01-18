use crate::{Alpm, Package, Result, SigLevel};

use alpm_sys::*;

use std::ffi::CString;
use std::os::raw::c_int;
use std::ptr;

impl Alpm {
    pub fn pkg_load<S: Into<String>>(
        &self,
        filename: S,
        full: bool,
        level: SigLevel,
    ) -> Result<Package> {
        let filename = CString::new(filename.into()).unwrap();
        let mut pkg = Package {
            pkg: ptr::null_mut(),
            handle: self,
            drop: true,
        };

        let ret = unsafe {
            alpm_pkg_load(
                self.handle,
                filename.as_ptr(),
                full as c_int,
                level.bits() as i32,
                &mut pkg.pkg,
            )
        };
        self.check_ret(ret)?;
        Ok(pkg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let pkg = handle
            .pkg_load(
                "tests/pacman-5.1.3-1-x86_64.pkg.tar.xz",
                false,
                SigLevel::NONE,
            )
            .unwrap();
        assert_eq!(pkg.name(), "pacman");
        assert_eq!(pkg.version(), "5.1.3-1");
        assert_eq!(pkg.base(), Some("pacman"));
        assert_eq!(
            pkg.desc(),
            Some("A library-based package manager with dependency support")
        );
        assert_eq!(pkg.url(), Some("https://www.archlinux.org/pacman/"));
        assert_eq!(pkg.packager(), Some("Allan McRae <allan@archlinux.org>"));
        assert_eq!(pkg.arch(), Some("x86_64"));
        assert_eq!(pkg.md5sum(), None);
        assert_eq!(pkg.sha256sum(), None);
        assert_eq!(pkg.base64_sig(), None);
    }

    #[test]
    fn load_incomplete() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let pkg = handle
            .pkg_load(
                "tests/pacman-5.1.3-1-incomplete.pkg.tar.xz",
                false,
                SigLevel::NONE,
            )
            .unwrap();
        assert_eq!(pkg.name(), "pacman");
        assert_eq!(pkg.version(), "5.1.3-1");
        assert_eq!(pkg.base(), None);
        assert_eq!(pkg.desc(), None);
        assert_eq!(pkg.url(), None);
        assert_eq!(pkg.packager(), None);
        assert_eq!(pkg.arch(), None);
        assert_eq!(pkg.md5sum(), None);
        assert_eq!(pkg.sha256sum(), None);
        assert_eq!(pkg.base64_sig(), None);
    }
}
