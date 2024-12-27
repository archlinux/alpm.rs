use crate::{Alpm, Pkg, Result, SigLevel};

use alpm_sys::*;

use std::ffi::CString;
use std::os::raw::c_int;
use std::ptr;

#[derive(Debug)]
pub struct LoadedPackage<'a> {
    pub(crate) pkg: &'a Pkg,
}

impl Drop for LoadedPackage<'_> {
    fn drop(&mut self) {
        unsafe {
            alpm_pkg_free(self.pkg.as_ptr());
        }
    }
}

impl std::ops::Deref for LoadedPackage<'_> {
    type Target = Pkg;

    fn deref(&self) -> &Self::Target {
        self.pkg
    }
}

impl AsRef<Pkg> for LoadedPackage<'_> {
    fn as_ref(&self) -> &Pkg {
        self
    }
}

impl<'a> LoadedPackage<'a> {
    pub(crate) unsafe fn from_ptr<'b>(pkg: *mut alpm_pkg_t) -> LoadedPackage<'b> {
        LoadedPackage {
            pkg: Pkg::from_ptr(pkg),
        }
    }

    pub fn pkg(&'a self) -> &'a Pkg {
        self.pkg
    }
}

impl Alpm {
    pub fn pkg_load<S: Into<Vec<u8>>>(
        &self,
        filename: S,
        full: bool,
        level: SigLevel,
    ) -> Result<LoadedPackage> {
        let filename = CString::new(filename).unwrap();
        let mut pkg = ptr::null_mut();

        let ret = unsafe {
            alpm_pkg_load(
                self.as_ptr(),
                filename.as_ptr(),
                full as c_int,
                level.bits() as i32,
                &mut pkg,
            )
        };
        self.check_ret(ret)?;
        Ok(LoadedPackage {
            pkg: unsafe { Pkg::from_ptr(pkg) },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    #[test]
    fn load() -> Result<()> {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let pkg = handle.pkg_load(
            "tests/pacman-5.1.3-1-x86_64.pkg.tar.xz",
            false,
            SigLevel::NONE,
        )?;
        assert_eq!(pkg.name(), "pacman");
        assert_eq!(pkg.version().as_str(), "5.1.3-1");
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

        Ok(())
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
        assert_eq!(pkg.version().as_str(), "5.1.3-1");
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
