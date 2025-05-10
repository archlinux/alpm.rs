use crate::utils::*;
use crate::{Alpm, AlpmList, AsAlpmList, Db, DbMut, Dep, Match, Result, SigLevel};

use alpm_sys::*;
use std::cmp::Ordering;
use std::ffi::CString;
use std::ptr;

impl Alpm {
    pub fn as_alpm_handle_t(&self) -> *mut alpm_handle_t {
        self.as_ptr()
    }

    pub fn unlock(&self) -> Result<()> {
        let ret = unsafe { alpm_unlock(self.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn root(&self) -> &str {
        unsafe { from_cstr(alpm_option_get_root(self.as_ptr())) }
    }

    pub fn dbpath(&self) -> &str {
        unsafe { from_cstr(alpm_option_get_dbpath(self.as_ptr())) }
    }

    pub fn hookdirs(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_hookdirs(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn cachedirs(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_cachedirs(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn lockfile(&self) -> &str {
        unsafe { from_cstr(alpm_option_get_lockfile(self.as_ptr())) }
    }

    pub fn gpgdir(&self) -> Option<&str> {
        unsafe { from_cstr_optional(alpm_option_get_gpgdir(self.as_ptr())) }
    }

    pub fn use_syslog(&self) -> bool {
        unsafe { alpm_option_get_usesyslog(self.as_ptr()) != 0 }
    }
    pub fn sandbox_user(&self) -> Option<&str> {
        unsafe { from_cstr_optional(alpm_option_get_sandboxuser(self.as_ptr())) }
    }

    pub fn noupgrades(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_noupgrades(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn noextracts(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_noextracts(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn ignorepkgs(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_ignorepkgs(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn ignoregroups(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_ignoregroups(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn overwrite_files(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_overwrite_files(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn assume_installed(&self) -> AlpmList<'_, &Dep> {
        let list = unsafe { alpm_option_get_assumeinstalled(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn architectures(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_architectures(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn check_space(&self) -> bool {
        unsafe { alpm_option_get_checkspace(self.as_ptr()) != 0 }
    }

    pub fn dbext(&self) -> &str {
        unsafe { from_cstr(alpm_option_get_dbext(self.as_ptr())) }
    }

    pub fn add_hookdir<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_hookdir(self.as_ptr(), s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_hookdirs<'a, T: AsAlpmList<&'a str>>(&mut self, list: T) -> Result<()> {
        list.with(|list| {
            let ret = unsafe { alpm_option_set_hookdirs(self.as_ptr(), list.as_ptr()) };
            self.check_ret(ret)
        })
    }

    pub fn remove_hookdir<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_hookdir(self.as_ptr(), s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn add_cachedir<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_cachedir(self.as_ptr(), s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_cachedirs<'a, T: AsAlpmList<&'a str>>(&mut self, list: T) -> Result<()> {
        list.with(|list| {
            let ret = unsafe { alpm_option_set_cachedirs(self.as_ptr(), list.as_ptr()) };
            self.check_ret(ret)
        })
    }

    pub fn remove_cachedir<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_cachedir(self.as_ptr(), s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn logfile(&self) -> Option<&str> {
        unsafe { from_cstr_optional(alpm_option_get_logfile(self.as_ptr())) }
    }

    pub fn set_logfile<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_set_logfile(self.as_ptr(), s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_gpgdir<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_set_gpgdir(self.as_ptr(), s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_use_syslog(&self, b: bool) {
        let b = if b { 1 } else { 0 };
        unsafe { alpm_option_set_usesyslog(self.as_ptr(), b) };
    }

    pub fn set_sandbox_user<S: Into<Vec<u8>>>(&mut self, s: Option<S>) -> Result<()> {
        let ret = if let Some(s) = s {
            let s = CString::new(s).unwrap();
            unsafe { alpm_option_set_sandboxuser(self.as_ptr(), s.as_ptr()) }
        } else {
            unsafe { alpm_option_set_sandboxuser(self.as_ptr(), ptr::null()) }
        };
        self.check_ret(ret)
    }

    pub fn set_disable_sandbox(&self, b: bool) {
        let b = if b { 1 } else { 0 };
        unsafe { alpm_option_set_disable_sandbox(self.as_ptr(), b) };
    }

    pub fn add_noupgrade<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_noupgrade(self.as_ptr(), s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_noupgrades<'a, T: AsAlpmList<&'a str>>(&mut self, list: T) -> Result<()> {
        list.with(|list| {
            let ret = unsafe { alpm_option_set_noupgrades(self.as_ptr(), list.as_ptr()) };
            self.check_ret(ret)
        })
    }

    pub fn remove_noupgrade<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_noupgrade(self.as_ptr(), s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn match_noupgrade<S: Into<Vec<u8>>>(&mut self, s: S) -> Match {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_match_noupgrade(self.as_ptr(), s.as_ptr()) };

        match ret.cmp(&0) {
            Ordering::Equal => Match::Yes,
            Ordering::Greater => Match::Inverted,
            Ordering::Less => Match::No,
        }
    }

    pub fn add_noextract<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_noextract(self.as_ptr(), s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_noextracts<'a, T: AsAlpmList<&'a str>>(&mut self, list: T) -> Result<()> {
        list.with(|list| {
            let ret = unsafe { alpm_option_set_noextracts(self.as_ptr(), list.as_ptr()) };
            self.check_ret(ret)
        })
    }

    pub fn remove_noextract<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_noextract(self.as_ptr(), s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn match_noextract<S: Into<Vec<u8>>>(&mut self, s: S) -> Match {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_match_noextract(self.as_ptr(), s.as_ptr()) };

        match ret.cmp(&0) {
            Ordering::Equal => Match::Yes,
            Ordering::Greater => Match::Inverted,
            Ordering::Less => Match::No,
        }
    }

    pub fn add_ignorepkg<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_ignorepkg(self.as_ptr(), s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_ignorepkgs<'a, T: AsAlpmList<&'a str>>(&mut self, list: T) -> Result<()> {
        list.with(|list| {
            let ret = unsafe { alpm_option_set_ignorepkgs(self.as_ptr(), list.as_ptr()) };
            self.check_ret(ret)
        })
    }

    pub fn remove_ignorepkg<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_ignorepkg(self.as_ptr(), s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn add_ignoregroup<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_ignoregroup(self.as_ptr(), s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_ignoregroups<'a, T: AsAlpmList<&'a str>>(&mut self, list: T) -> Result<()> {
        list.with(|list| {
            let ret = unsafe { alpm_option_set_ignoregroups(self.as_ptr(), list.as_ptr()) };
            self.check_ret(ret)
        })
    }

    pub fn remove_ignoregroup<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_ignoregroup(self.as_ptr(), s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn add_overwrite_file<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_overwrite_file(self.as_ptr(), s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_overwrite_files<'a, T: AsAlpmList<&'a str>>(&mut self, list: T) -> Result<()> {
        list.with(|list| {
            let ret = unsafe { alpm_option_set_overwrite_files(self.as_ptr(), list.as_ptr()) };
            self.check_ret(ret)
        })
    }

    pub fn remove_overwrite_file<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_overwrite_file(self.as_ptr(), s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn add_assume_installed(&mut self, s: &Dep) -> Result<()> {
        let ret = unsafe { alpm_option_add_assumeinstalled(self.as_ptr(), s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_assume_installed<'a, T: AsAlpmList<&'a Dep>>(&mut self, list: T) -> Result<()> {
        list.with(|list| {
            let ret = unsafe { alpm_option_set_assumeinstalled(self.as_ptr(), list.as_ptr()) };
            self.check_ret(ret)
        })
    }

    pub fn remove_assume_installed(&mut self, s: &Dep) -> Result<bool> {
        let ret = unsafe { alpm_option_remove_assumeinstalled(self.as_ptr(), s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn add_architecture<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_architecture(self.as_ptr(), s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_architectures<'a, T: AsAlpmList<&'a str>>(&mut self, list: T) -> Result<()> {
        list.with(|list| {
            let ret = unsafe { alpm_option_set_architectures(self.as_ptr(), list.as_ptr()) };
            self.check_ret(ret)
        })
    }

    pub fn remove_architecture<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_architecture(self.as_ptr(), s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn localdb(&self) -> &Db {
        let db = unsafe { alpm_get_localdb(self.as_ptr()) };
        unsafe { Db::from_ptr(db) }
    }

    pub fn syncdbs(&self) -> AlpmList<&Db> {
        let dbs = unsafe { alpm_get_syncdbs(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(dbs) }
    }

    pub fn syncdbs_mut(&mut self) -> AlpmList<DbMut> {
        let dbs = unsafe { alpm_get_syncdbs(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(dbs) }
    }

    pub fn set_check_space(&self, b: bool) {
        let b = if b { 1 } else { 0 };
        unsafe { alpm_option_set_checkspace(self.as_ptr(), b) };
    }

    pub fn set_dbext<S: Into<Vec<u8>>>(&mut self, s: S) {
        let s = CString::new(s).unwrap();
        unsafe { alpm_option_set_dbext(self.as_ptr(), s.as_ptr()) };
    }

    pub fn set_default_siglevel(&self, s: SigLevel) -> Result<()> {
        let ret = unsafe { alpm_option_set_default_siglevel(self.as_ptr(), s.bits() as i32) };
        self.check_ret(ret)
    }

    pub fn default_siglevel(&self) -> SigLevel {
        let ret = unsafe { alpm_option_get_default_siglevel(self.as_ptr()) };
        SigLevel::from_bits(ret as u32).unwrap()
    }

    pub fn set_local_file_siglevel(&self, s: SigLevel) -> Result<()> {
        let ret = unsafe { alpm_option_set_local_file_siglevel(self.as_ptr(), s.bits() as i32) };
        self.check_ret(ret)
    }

    pub fn local_file_siglevel(&self) -> SigLevel {
        let ret = unsafe { alpm_option_get_local_file_siglevel(self.as_ptr()) };
        SigLevel::from_bits(ret as u32).unwrap()
    }

    pub fn set_remote_file_siglevel(&self, s: SigLevel) -> Result<()> {
        let ret = unsafe { alpm_option_set_remote_file_siglevel(self.as_ptr(), s.bits() as i32) };
        self.check_ret(ret)
    }

    pub fn remote_file_siglevel(&self) -> SigLevel {
        let ret = unsafe { alpm_option_get_remote_file_siglevel(self.as_ptr()) };
        SigLevel::from_bits(ret as u32).unwrap()
    }

    pub fn set_disable_dl_timeout(&self, b: bool) {
        let b = if b { 1 } else { 0 };
        unsafe { alpm_option_set_disable_dl_timeout(self.as_ptr(), b) };
    }

    #[cfg(feature = "git")]
    #[doc(alias = "disable_dl_timeout")]
    pub fn dl_timeout_disabled(&self) -> bool {
        unsafe { alpm_option_get_disable_dl_timeout(self.as_ptr()) != 0 }
    }

    pub fn set_parallel_downloads(&self, n: u32) {
        unsafe { alpm_option_set_parallel_downloads(self.as_ptr(), n) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_getters() {
        let handle = Alpm::new("/", "tests/db/").unwrap();

        assert_eq!(handle.root(), "/");
        assert!(handle.dbpath().ends_with("tests/db/"));

        assert!(handle.cachedirs().is_empty());
        assert!(!handle.lockfile().is_empty());
        assert!(!handle.use_syslog());
        assert!(handle.assume_installed().is_empty());
        assert!(!handle.dbext().is_empty());
        assert!(handle.gpgdir().is_none());
        assert!(handle.logfile().is_none());
    }

    #[test]
    fn test_setters() {
        let mut handle = Alpm::new("/", "tests/db/").unwrap();

        handle.set_hookdirs(["1", "2", "3"].iter()).unwrap();
        handle.add_hookdir("x").unwrap();
        handle.set_hookdirs(["a", "b", "c"].iter()).unwrap();
        handle.add_hookdir("z").unwrap();
        let hooks = handle.hookdirs().iter().collect::<Vec<_>>();
        assert_eq!(hooks, vec!["a/", "b/", "c/", "z/"]);

        assert!(!handle.check_space());
        handle.set_check_space(true);
        assert!(handle.check_space());
        handle.set_check_space(false);
        assert!(!handle.check_space());

        assert_eq!(handle.default_siglevel(), SigLevel::NONE);
        if crate::Capabilities::new().signatures() {
            handle
                .set_default_siglevel(SigLevel::PACKAGE | SigLevel::DATABASE)
                .unwrap();
            assert_eq!(
                handle.default_siglevel(),
                SigLevel::PACKAGE | SigLevel::DATABASE
            );
        }

        handle.set_ignorepkgs(["a", "b", "c"].iter()).unwrap();
        let pkgs = handle.ignorepkgs().iter().collect::<Vec<_>>();
        assert_eq!(pkgs.as_slice(), ["a", "b", "c"]);

        /*let indeps = vec!["a", "b", "c"].into_iter().map(|s| Depend::new(s)).collect::<Vec<_>>();
        let deps = vec!["a", "b", "c"].into_iter().map(|s| Depend::new(s)).collect::<Vec<_>>();
        handle.set_assume_installed(indeps);

        let ai = handle.assume_installed().collect::<Vec<_>>();
        assert_eq!(deps.into_iter().map(|d| d.to_string()).collect::<Vec<_>>(), ai.into_iter().map(|d| d.to_string()).collect::<Vec<_>>());
        */
    }
}
