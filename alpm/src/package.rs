use crate::utils::*;
use crate::{
    AlpmList, AlpmListMut, Backup, ChangeLog, Db, Dep, Error, FileList, PackageFrom, PackageReason,
    PackageValidation, Result, Signature, Ver,
};

#[cfg(feature = "mtree")]
use crate::MTree;

use std::cell::UnsafeCell;
use std::mem::transmute;
use std::ops::Deref;
use std::os::raw::c_int;
use std::{fmt, ptr};

use alpm_sys::*;

#[repr(transparent)]
pub struct Package {
    pkg: UnsafeCell<alpm_pkg_t>,
}

#[repr(transparent)]
pub struct Pkg {
    pkg: UnsafeCell<alpm_pkg_t>,
}

impl fmt::Debug for Pkg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pkg")
            .field("name", &self.name())
            .field("version", &self.version())
            .finish()
    }
}

impl fmt::Debug for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Package")
            .field("name", &self.name())
            .field("version", &self.version())
            .finish()
    }
}

impl Deref for Package {
    type Target = Pkg;
    fn deref(&self) -> &Self::Target {
        unsafe { Pkg::from_ptr(self.pkg.get()) }
    }
}

impl AsRef<Pkg> for Pkg {
    fn as_ref(&self) -> &Pkg {
        self
    }
}

impl AsRef<Pkg> for Package {
    fn as_ref(&self) -> &Pkg {
        self
    }
}

impl Package {
    pub(crate) unsafe fn from_ptr<'a>(pkg: *mut alpm_pkg_t) -> &'a Package {
        &*(pkg as *mut Package)
    }
}

impl Pkg {
    pub(crate) unsafe fn from_ptr<'a>(pkg: *mut alpm_pkg_t) -> &'a Pkg {
        &*(pkg as *mut Pkg)
    }

    pub(crate) fn handle_ptr(&self) -> *mut alpm_handle_t {
        unsafe { alpm_pkg_get_handle(self.as_ptr()) }
    }

    pub(crate) fn as_ptr(&self) -> *mut alpm_pkg_t {
        self.pkg.get()
    }

    pub(crate) fn last_error(&self) -> Error {
        unsafe { Error::new(alpm_errno(self.handle_ptr())) }
    }

    pub(crate) fn check_ret(&self, int: c_int) -> Result<()> {
        if int != 0 {
            Err(self.last_error())
        } else {
            Ok(())
        }
    }

    pub(crate) fn check_null<T>(&self, ptr: *const T) -> Result<()> {
        if ptr.is_null() {
            Err(self.last_error())
        } else {
            Ok(())
        }
    }

    pub fn name(&self) -> &str {
        let name = unsafe { alpm_pkg_get_name(self.as_ptr()) };
        unsafe { from_cstr(name) }
    }

    pub fn check_md5sum(&self) -> Result<()> {
        self.check_ret(unsafe { alpm_pkg_checkmd5sum(self.as_ptr()) })
    }

    pub fn should_ignore(&self) -> bool {
        let ret = unsafe { alpm_pkg_should_ignore(self.handle_ptr(), self.as_ptr()) };
        ret != 0
    }

    pub fn filename(&self) -> Option<&str> {
        let name = unsafe { alpm_pkg_get_filename(self.as_ptr()) };
        unsafe { from_cstr_optional(name) }
    }

    pub fn base(&self) -> Option<&str> {
        let base = unsafe { alpm_pkg_get_base(self.as_ptr()) };
        unsafe { from_cstr_optional(base) }
    }

    pub fn version(&self) -> &Ver {
        let version = unsafe { alpm_pkg_get_version(self.as_ptr()) };
        unsafe { Ver::from_ptr(version) }
    }

    pub fn origin(&self) -> PackageFrom {
        let origin = unsafe { alpm_pkg_get_origin(self.as_ptr()) };
        unsafe { transmute::<_alpm_pkgfrom_t, PackageFrom>(origin) }
    }

    pub fn desc(&self) -> Option<&str> {
        let desc = unsafe { alpm_pkg_get_desc(self.as_ptr()) };
        unsafe { from_cstr_optional(desc) }
    }

    pub fn url(&self) -> Option<&str> {
        let url = unsafe { alpm_pkg_get_url(self.as_ptr()) };
        unsafe { from_cstr_optional(url) }
    }

    pub fn build_date(&self) -> i64 {
        let date = unsafe { alpm_pkg_get_builddate(self.as_ptr()) };
        date as i64
    }

    pub fn install_date(&self) -> Option<i64> {
        let date = unsafe { alpm_pkg_get_installdate(self.as_ptr()) };
        if date == 0 {
            None
        } else {
            Some(date as i64)
        }
    }

    pub fn packager(&self) -> Option<&str> {
        let packager = unsafe { alpm_pkg_get_packager(self.as_ptr()) };
        unsafe { from_cstr_optional(packager) }
    }

    pub fn md5sum(&self) -> Option<&str> {
        let md5sum = unsafe { alpm_pkg_get_md5sum(self.as_ptr()) };
        unsafe { from_cstr_optional(md5sum) }
    }

    pub fn sha256sum(&self) -> Option<&str> {
        let sha256sum = unsafe { alpm_pkg_get_sha256sum(self.as_ptr()) };
        unsafe { from_cstr_optional(sha256sum) }
    }

    pub fn arch(&self) -> Option<&str> {
        let arch = unsafe { alpm_pkg_get_arch(self.as_ptr()) };
        unsafe { from_cstr_optional(arch) }
    }

    pub fn size(&self) -> i64 {
        let size = unsafe { alpm_pkg_get_size(self.as_ptr()) };
        size as i64
    }

    pub fn isize(&self) -> i64 {
        let size = unsafe { alpm_pkg_get_isize(self.as_ptr()) };
        size as i64
    }

    pub fn reason(&self) -> PackageReason {
        let reason = unsafe { alpm_pkg_get_reason(self.as_ptr()) };
        unsafe { transmute::<_alpm_pkgreason_t, PackageReason>(reason) }
    }

    pub fn validation(&self) -> PackageValidation {
        let validation = unsafe { alpm_pkg_get_validation(self.as_ptr()) };
        PackageValidation::from_bits(validation as u32).unwrap()
    }

    pub fn licenses(&self) -> AlpmList<&str> {
        let list = unsafe { alpm_pkg_get_licenses(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn groups(&self) -> AlpmList<&str> {
        let list = unsafe { alpm_pkg_get_groups(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn depends(&self) -> AlpmList<&Dep> {
        let list = unsafe { alpm_pkg_get_depends(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn optdepends(&self) -> AlpmList<&Dep> {
        let list = unsafe { alpm_pkg_get_optdepends(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn checkdepends(&self) -> AlpmList<&Dep> {
        let list = unsafe { alpm_pkg_get_checkdepends(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn makedepends(&self) -> AlpmList<&Dep> {
        let list = unsafe { alpm_pkg_get_makedepends(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn conflicts(&self) -> AlpmList<&Dep> {
        let list = unsafe { alpm_pkg_get_conflicts(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn provides(&self) -> AlpmList<&Dep> {
        let list = unsafe { alpm_pkg_get_provides(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn replaces(&self) -> AlpmList<&Dep> {
        let list = unsafe { alpm_pkg_get_replaces(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn files(&self) -> FileList {
        let files = unsafe { *alpm_pkg_get_files(self.as_ptr()) };
        unsafe { FileList::new(files) }
    }

    pub fn backup(&self) -> AlpmList<&Backup> {
        let list = unsafe { alpm_pkg_get_backup(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn db(&self) -> Option<&Db> {
        let db = unsafe { alpm_pkg_get_db(self.as_ptr()) };
        self.check_null(db).ok()?;
        unsafe { Some(Db::from_ptr(db)) }
    }

    pub fn changelog(&self) -> Result<ChangeLog> {
        let changelog = unsafe { alpm_pkg_changelog_open(self.as_ptr()) };
        self.check_null(changelog)?;
        let changelog = unsafe { ChangeLog::new(self, changelog) };
        Ok(changelog)
    }

    #[cfg(feature = "mtree")]
    pub fn mtree(&self) -> Result<MTree> {
        let archive = unsafe { alpm_pkg_mtree_open(self.as_ptr()) };
        self.check_null(archive)?;

        let archive = unsafe { MTree::new(self, archive) };

        Ok(archive)
    }

    pub fn required_by(&self) -> AlpmListMut<String> {
        let list = unsafe { alpm_pkg_compute_requiredby(self.as_ptr()) };
        unsafe { AlpmListMut::from_ptr(list) }
    }

    pub fn optional_for(&self) -> AlpmListMut<String> {
        let list = unsafe { alpm_pkg_compute_optionalfor(self.as_ptr()) };
        unsafe { AlpmListMut::from_ptr(list) }
    }

    pub fn base64_sig(&self) -> Option<&str> {
        let base64_sig = unsafe { alpm_pkg_get_base64_sig(self.as_ptr()) };
        unsafe { from_cstr_optional(base64_sig) }
    }

    pub fn has_scriptlet(&self) -> bool {
        unsafe { alpm_pkg_has_scriptlet(self.as_ptr()) != 0 }
    }

    pub fn sig(&self) -> Result<Signature> {
        let mut sig = ptr::null_mut();
        let mut len = 0;
        let ret = unsafe { alpm_pkg_get_sig(self.as_ptr(), &mut sig, &mut len) };
        self.check_ret(ret)?;
        let sig = unsafe { Signature::new(sig, len) };
        Ok(sig)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Alpm, SigLevel};
    use std::io::Read;
    use std::mem::size_of;

    #[test]
    fn test_depends() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let depends = pkg
            .depends()
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            &depends,
            &["coreutils", "linux-firmware", "kmod", "mkinitcpio"]
        )
    }

    #[test]
    fn test_files() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.localdb();
        let pkg = db.pkg("filesystem").unwrap();
        let files = pkg.files();

        for file in files.files() {
            println!("{}", file.name());
        }

        assert!(files.contains("etc/").is_some());
        assert!(pkg.filename().is_none());
    }

    #[test]
    fn test_files_null() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("filesystem").unwrap();
        let files = pkg.files();

        assert!(files.files().is_empty());
    }

    #[test]
    fn test_groups() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let groups = pkg.groups();

        assert_eq!(&groups.iter().collect::<Vec<_>>(), &["base"],)
    }

    #[test]
    fn test_backup() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.localdb();
        let pkg = db.pkg("pacman").unwrap();
        let backup = pkg.backup();
        assert_eq!(backup.first().unwrap().name(), "etc/pacman.conf");
    }

    #[test]
    fn test_rquired_by() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("extra", SigLevel::NONE).unwrap();
        let pkg = db.pkg("ostree").unwrap();
        let optional = pkg
            .required_by()
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>();
        assert_eq!(&optional, &["flatpak"]);
    }

    #[test]
    fn test_changelog() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.localdb();
        let pkg = db.pkg("vifm").unwrap();
        let mut s = String::new();
        let mut changelog = pkg.changelog().unwrap();
        changelog.read_to_string(&mut s).unwrap();
        assert!(s.contains("2010-02-15 Jaroslav Lichtblau <svetlemodry@archlinux.org>"));
    }

    #[test]
    fn test_pkg_optimization() {
        assert!(size_of::<&Pkg>() == size_of::<&Option<Pkg>>());
    }

    #[test]
    fn test_lifetime() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("extra", SigLevel::NONE).unwrap();
        let pkg = db.pkg("ostree").unwrap();
        //drop(handle);
        println!("{}", pkg.name());
    }
}
