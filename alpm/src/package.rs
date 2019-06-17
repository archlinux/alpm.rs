use crate::utils::*;
use crate::{
    Alpm, AlpmList, ChangeLog, Db, Depend, FileList, FreeMethod, PackageFrom, PackageReason,
    PackageValidation, Result,
};

#[cfg(feature = "mtree")]
use crate::MTree;

use std::mem::transmute;

use alpm_sys::*;

#[derive(Debug)]
pub struct Package<'a> {
    pub(crate) handle: &'a Alpm,
    pub(crate) pkg: *mut alpm_pkg_t,
    pub(crate) drop: bool,
}

impl<'a> Drop for Package<'a> {
    fn drop(&mut self) {
        if self.drop {
            unsafe { alpm_pkg_free(self.pkg) };
        }
    }
}

impl<'a> Package<'a> {
    pub fn name(&self) -> &'a str {
        let name = unsafe { alpm_pkg_get_name(self.pkg) };
        unsafe { from_cstr(name) }
    }

    pub fn check_md5sum(&self) -> Result<()> {
        self.handle
            .check_ret(unsafe { alpm_pkg_checkmd5sum(self.pkg) })
    }

    pub fn should_ignore(&self) -> bool {
        let ret = unsafe { alpm_pkg_should_ignore(self.handle.handle, self.pkg) };
        ret != 0
    }

    pub fn filename(&self) -> &'a str {
        let name = unsafe { alpm_pkg_get_filename(self.pkg) };
        unsafe { from_cstr(name) }
    }

    pub fn base(&self) -> &'a str {
        let base = unsafe { alpm_pkg_get_base(self.pkg) };
        unsafe { from_cstr(base) }
    }

    pub fn version(&self) -> &'a str {
        let version = unsafe { alpm_pkg_get_version(self.pkg) };
        unsafe { from_cstr(version) }
    }

    pub fn origin(&self) -> PackageFrom {
        let origin = unsafe { alpm_pkg_get_origin(self.pkg) };
        unsafe { transmute::<_alpm_pkgfrom_t, PackageFrom>(origin) }
    }

    pub fn desc(&self) -> &'a str {
        let desc = unsafe { alpm_pkg_get_desc(self.pkg) };
        unsafe { from_cstr(desc) }
    }

    pub fn url(&self) -> &'a str {
        let url = unsafe { alpm_pkg_get_url(self.pkg) };
        unsafe { from_cstr(url) }
    }

    pub fn build_date(&self) -> i64 {
        let date = unsafe { alpm_pkg_get_builddate(self.pkg) };
        date as i64
    }

    pub fn install_date(&self) -> Option<i64> {
        let date = unsafe { alpm_pkg_get_installdate(self.pkg) };
        if date == 0 {
            None
        } else {
            Some(date as i64)
        }
    }

    pub fn packager(&self) -> &'a str {
        let packager = unsafe { alpm_pkg_get_packager(self.pkg) };
        unsafe { from_cstr(packager) }
    }

    pub fn md5sum(&self) -> &'a str {
        let md5sum = unsafe { alpm_pkg_get_md5sum(self.pkg) };
        unsafe { from_cstr(md5sum) }
    }

    pub fn sha256sum(&self) -> &'a str {
        let sha256sum = unsafe { alpm_pkg_get_sha256sum(self.pkg) };
        unsafe { from_cstr(sha256sum) }
    }

    pub fn arch(&self) -> &'a str {
        let arch = unsafe { alpm_pkg_get_arch(self.pkg) };
        unsafe { from_cstr(arch) }
    }

    pub fn size(&self) -> i64 {
        let size = unsafe { alpm_pkg_get_size(self.pkg) };
        size as i64
    }

    pub fn isize(&self) -> i64 {
        let size = unsafe { alpm_pkg_get_isize(self.pkg) };
        size as i64
    }

    pub fn reason(&self) -> PackageReason {
        let reason = unsafe { alpm_pkg_get_reason(self.pkg) };
        unsafe { transmute::<_alpm_pkgreason_t, PackageReason>(reason) }
    }

    pub fn validation(&self) -> PackageValidation {
        let validation = unsafe { alpm_pkg_get_validation(self.pkg) };
        PackageValidation::from_bits(validation as u32).unwrap()
    }

    pub fn licenses(&self) -> AlpmList<'a, &'a str> {
        let list = unsafe { alpm_pkg_get_licenses(self.pkg) };
        AlpmList::new(self.handle, list, FreeMethod::None)
    }

    pub fn groups(&self) -> AlpmList<'a, &'a str> {
        let list = unsafe { alpm_pkg_get_groups(self.pkg) };
        AlpmList::new(self.handle, list, FreeMethod::None)
    }

    pub fn depends(&self) -> AlpmList<'a, Depend> {
        let list = unsafe { alpm_pkg_get_depends(self.pkg) };
        AlpmList::new(self.handle, list, FreeMethod::None)
    }

    pub fn optdepends(&self) -> AlpmList<'a, Depend> {
        let list = unsafe { alpm_pkg_get_optdepends(self.pkg) };
        AlpmList::new(self.handle, list, FreeMethod::None)
    }

    pub fn checkdepends(&self) -> AlpmList<'a, Depend> {
        let list = unsafe { alpm_pkg_get_checkdepends(self.pkg) };
        AlpmList::new(self.handle, list, FreeMethod::None)
    }

    pub fn makedepends(&self) -> AlpmList<'a, Depend> {
        let list = unsafe { alpm_pkg_get_makedepends(self.pkg) };
        AlpmList::new(self.handle, list, FreeMethod::None)
    }

    pub fn conflicts(&self) -> AlpmList<'a, Depend> {
        let list = unsafe { alpm_pkg_get_conflicts(self.pkg) };
        AlpmList::new(self.handle, list, FreeMethod::None)
    }

    pub fn provides(&self) -> AlpmList<'a, Depend> {
        let list = unsafe { alpm_pkg_get_provides(self.pkg) };
        AlpmList::new(self.handle, list, FreeMethod::None)
    }

    pub fn replaces(&self) -> AlpmList<'a, Depend> {
        let list = unsafe { alpm_pkg_get_replaces(self.pkg) };
        AlpmList::new(self.handle, list, FreeMethod::None)
    }

    pub fn files(&self) -> FileList {
        let files = unsafe { *alpm_pkg_get_files(self.pkg) };
        FileList { inner: files }
    }

    pub fn backup(&self) -> AlpmList<'a, &'a str> {
        let list = unsafe { alpm_pkg_get_backup(self.pkg) };
        AlpmList::new(self.handle, list, FreeMethod::None)
    }

    pub fn db(&self) -> Result<Db> {
        let db = unsafe { alpm_pkg_get_db(self.pkg) };
        self.handle.check_null(db)?;
        Ok(Db {
            handle: self.handle,
            db,
        })
    }

    pub fn changelog(&self) -> Result<ChangeLog> {
        let changelog = unsafe { alpm_pkg_changelog_open(self.pkg) };
        self.handle.check_null(changelog)?;

        let changelog = ChangeLog {
            pkg: self,
            stream: changelog,
        };

        Ok(changelog)
    }

    #[cfg(feature = "mtree")]
    pub fn mtree(&self) -> Result<MTree> {
        let archive = unsafe { alpm_pkg_mtree_open(self.pkg) };
        self.handle.check_null(archive)?;

        let archive = MTree { pkg: self, archive };

        Ok(archive)
    }

    pub fn required_by(&self) -> AlpmList<'a, &'a str> {
        let list = unsafe { alpm_pkg_compute_requiredby(self.pkg) };
        AlpmList::new(self.handle, list, FreeMethod::FreeInner)
    }

    pub fn optional_for(&self) -> AlpmList<'a, &'a str> {
        let list = unsafe { alpm_pkg_compute_optionalfor(self.pkg) };
        AlpmList::new(self.handle, list, FreeMethod::FreeInner)
    }

    pub fn base64_sig(&self) -> &'a str {
        let base64_sig = unsafe { alpm_pkg_get_base64_sig(self.pkg) };
        unsafe { from_cstr(base64_sig) }
    }

    pub fn has_scriptlet(&self) -> bool {
        unsafe { alpm_pkg_has_scriptlet(self.pkg) != 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SigLevel;
    use std::io::Read;

    #[test]
    fn test_depends() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let depends = pkg.depends().map(|d| d.to_string()).collect::<Vec<_>>();
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

        assert!(files.contains("etc/").unwrap().is_some());
    }

    #[test]
    fn test_groups() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let groups = pkg.groups();

        assert_eq!(&groups.collect::<Vec<_>>(), &["base"],)
    }

    #[test]
    fn test_rquired_by() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("extra", SigLevel::NONE).unwrap();
        let pkg = db.pkg("ostree").unwrap();
        let optional = pkg.required_by().map(|d| d.to_string()).collect::<Vec<_>>();
        assert_eq!(&optional, &["flatpak"])
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
    #[cfg(feature = "mtree")]
    fn test_mtree() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.localdb();
        let pkg = db.pkg("vifm").unwrap();
        let mtree = pkg.mtree().unwrap();

        println!("entries:");
        assert!(mtree.count() > 10);
    }

}
