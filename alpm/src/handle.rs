use crate::utils::*;
use crate::{free, Alpm, AlpmList, Db, DbMut, Dep, Depend, Match, Result, SigLevel};

use std::cmp::Ordering;
use std::ffi::CString;

use alpm_sys::*;

impl Alpm {
    pub fn as_alpm_handle_t(&self) -> *mut alpm_handle_t {
        self.handle
    }

    pub fn unlock(&self) -> Result<()> {
        let ret = unsafe { alpm_unlock(self.handle) };
        self.check_ret(ret)
    }

    pub fn root(&self) -> &str {
        unsafe { from_cstr(alpm_option_get_root(self.handle)) }
    }

    pub fn dbpath(&self) -> &str {
        unsafe { from_cstr(alpm_option_get_dbpath(self.handle)) }
    }

    pub fn hookdirs(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_hookdirs(self.handle) };
        AlpmList::from_parts(self, list)
    }

    pub fn cachedirs(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_cachedirs(self.handle) };
        AlpmList::from_parts(self, list)
    }

    pub fn lockfile(&self) -> &str {
        unsafe { from_cstr(alpm_option_get_lockfile(self.handle)) }
    }

    pub fn gpgdir(&self) -> &str {
        unsafe { from_cstr(alpm_option_get_gpgdir(self.handle)) }
    }

    pub fn use_syslog(&self) -> bool {
        unsafe { alpm_option_get_usesyslog(self.handle) != 0 }
    }

    pub fn noupgrades(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_noupgrades(self.handle) };
        AlpmList::from_parts(self, list)
    }

    pub fn noextracts(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_noextracts(self.handle) };
        AlpmList::from_parts(self, list)
    }

    pub fn ignorepkgs(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_ignorepkgs(self.handle) };
        AlpmList::from_parts(self, list)
    }

    pub fn ignoregroups(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_ignoregroups(self.handle) };
        AlpmList::from_parts(self, list)
    }

    pub fn overwrite_files(&self) -> AlpmList<'_, &str> {
        let list = unsafe { alpm_option_get_overwrite_files(self.handle) };
        AlpmList::from_parts(self, list)
    }

    pub fn assume_installed(&self) -> AlpmList<'_, Depend> {
        let list = unsafe { alpm_option_get_assumeinstalled(self.handle) };
        AlpmList::from_parts(self, list)
    }

    pub fn arch(&self) -> &str {
        unsafe { from_cstr(alpm_option_get_arch(self.handle)) }
    }

    pub fn check_space(&self) -> bool {
        unsafe { alpm_option_get_checkspace(self.handle) != 0 }
    }

    pub fn dbext(&self) -> &str {
        unsafe { from_cstr(alpm_option_get_dbext(self.handle)) }
    }

    pub fn add_hookdir<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_hookdir(self.handle, s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_hookdirs<S: Into<Vec<u8>>, I: IntoIterator<Item = S>>(
        &mut self,
        list: I,
    ) -> Result<()> {
        let list = to_strlist(list);
        let ret = unsafe { alpm_option_set_hookdirs(self.handle, list) };
        unsafe { alpm_list_free_inner(list, Some(free)) };
        unsafe { alpm_list_free(list) };
        self.check_ret(ret)
    }

    pub fn remove_hookdir<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_hookdir(self.handle, s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn add_cachedir<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_cachedir(self.handle, s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_cachedirs<S: Into<Vec<u8>>, I: IntoIterator<Item = S>>(
        &mut self,
        list: I,
    ) -> Result<()> {
        let list = to_strlist(list);
        let ret = unsafe { alpm_option_set_cachedirs(self.handle, list) };
        unsafe { alpm_list_free_inner(list, Some(free)) };
        unsafe { alpm_list_free(list) };
        self.check_ret(ret)
    }

    pub fn remove_cachedir<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_cachedir(self.handle, s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn set_logfile<S: Into<Vec<u8>>>(&self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_set_logfile(self.handle, s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_gpgdir<S: Into<Vec<u8>>>(&self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_set_gpgdir(self.handle, s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_use_syslog(&self, b: bool) {
        let b = if b { 1 } else { 0 };
        unsafe { alpm_option_set_usesyslog(self.handle, b) };
    }

    pub fn add_noupgrade<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_noupgrade(self.handle, s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_noupgrades<S: Into<Vec<u8>>, I: IntoIterator<Item = S>>(
        &mut self,
        list: I,
    ) -> Result<()> {
        let list = to_strlist(list);
        let ret = unsafe { alpm_option_set_noupgrades(self.handle, list) };
        unsafe { alpm_list_free_inner(list, Some(free)) };
        unsafe { alpm_list_free(list) };
        self.check_ret(ret)
    }

    pub fn remove_noupgrade<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_noupgrade(self.handle, s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn match_noupgrade<S: Into<Vec<u8>>>(&mut self, s: S) -> Match {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_match_noupgrade(self.handle, s.as_ptr()) };

        match ret.cmp(&0) {
            Ordering::Equal => Match::Yes,
            Ordering::Greater => Match::Inverted,
            Ordering::Less => Match::No,
        }
    }

    pub fn add_noextract<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_noextract(self.handle, s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_noextracts<S: Into<Vec<u8>>, I: IntoIterator<Item = S>>(
        &mut self,
        list: I,
    ) -> Result<()> {
        let list = to_strlist(list);
        let ret = unsafe { alpm_option_set_noextracts(self.handle, list) };
        unsafe { alpm_list_free_inner(list, Some(free)) };
        unsafe { alpm_list_free(list) };
        self.check_ret(ret)
    }

    pub fn remove_noextract<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_noextract(self.handle, s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn match_noextract<S: Into<Vec<u8>>>(&mut self, s: S) -> Match {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_match_noextract(self.handle, s.as_ptr()) };

        match ret.cmp(&0) {
            Ordering::Equal => Match::Yes,
            Ordering::Greater => Match::Inverted,
            Ordering::Less => Match::No,
        }
    }

    pub fn add_ignorepkg<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_ignorepkg(self.handle, s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_ignorepkgs<S: Into<Vec<u8>>, I: IntoIterator<Item = S>>(
        &mut self,
        list: I,
    ) -> Result<()> {
        let list = to_strlist(list);
        let ret = unsafe { alpm_option_set_ignorepkgs(self.handle, list) };
        unsafe { alpm_list_free_inner(list, Some(free)) };
        unsafe { alpm_list_free(list) };
        self.check_ret(ret)
    }

    pub fn remove_ignorepkg<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_ignorepkg(self.handle, s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn add_ignoregroup<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_ignoregroup(self.handle, s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_ignoregroups<S: Into<Vec<u8>>, I: IntoIterator<Item = S>>(
        &mut self,
        list: I,
    ) -> Result<()> {
        let list = to_strlist(list);
        let ret = unsafe { alpm_option_set_ignoregroups(self.handle, list) };
        unsafe { alpm_list_free_inner(list, Some(free)) };
        unsafe { alpm_list_free(list) };
        self.check_ret(ret)
    }

    pub fn remove_ignoregroup<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_ignoregroup(self.handle, s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn add_overwrite_file<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<()> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_add_overwrite_file(self.handle, s.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_overwrite_files<S: Into<Vec<u8>>, I: IntoIterator<Item = S>>(
        &mut self,
        list: I,
    ) -> Result<()> {
        let list = to_strlist(list);
        let ret = unsafe { alpm_option_set_overwrite_files(self.handle, list) };
        unsafe { alpm_list_free_inner(list, Some(free)) };
        unsafe { alpm_list_free(list) };
        self.check_ret(ret)
    }

    pub fn remove_overwrite_file<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<bool> {
        let s = CString::new(s).unwrap();
        let ret = unsafe { alpm_option_remove_overwrite_file(self.handle, s.as_ptr()) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn add_assume_installed<'a, D: AsRef<Dep<'a>>>(&mut self, s: D) -> Result<()> {
        let ret = unsafe { alpm_option_add_assumeinstalled(self.handle, s.as_ref().inner) };
        self.check_ret(ret)
    }

    // Broken in stable
    #[cfg(feature = "git")]
    pub fn set_assume_installed<'a, 'b, D: AsRef<Dep<'b>>, I: IntoIterator<Item = D>>(
        &'a mut self,
        list: I,
    ) -> Result<()> {
        let mut deps = std::ptr::null_mut();
        for dep in list.into_iter() {
            unsafe { deps = alpm_list_add(deps, dep.as_ref().inner as _) };
        }

        let ret = unsafe { alpm_option_set_assumeinstalled(self.handle, deps) };
        unsafe { alpm_list_free(deps) };
        self.check_ret(ret)
    }

    pub fn remove_assume_installed<'a, D: AsRef<Dep<'a>>>(&mut self, s: D) -> Result<bool> {
        let ret = unsafe { alpm_option_remove_assumeinstalled(self.handle, s.as_ref().inner) };
        if ret == 1 {
            Ok(true)
        } else {
            self.check_ret(ret).map(|_| false)
        }
    }

    pub fn set_arch<S: Into<Vec<u8>>>(&self, s: S) {
        let s = CString::new(s).unwrap();
        unsafe { alpm_option_set_arch(self.handle, s.as_ptr()) };
    }

    pub fn localdb(&self) -> Db {
        let db = unsafe { alpm_get_localdb(self.handle) };
        Db { handle: self, db }
    }

    pub fn syncdbs(&self) -> AlpmList<Db> {
        let dbs = unsafe { alpm_get_syncdbs(self.handle) };
        AlpmList::from_parts(self, dbs)
    }

    pub fn syncdbs_mut(&mut self) -> AlpmList<DbMut> {
        let dbs = unsafe { alpm_get_syncdbs(self.handle) };
        AlpmList::from_parts(self, dbs)
    }

    pub fn set_check_space(&self, b: bool) {
        let b = if b { 1 } else { 0 };
        unsafe { alpm_option_set_checkspace(self.handle, b) };
    }

    pub fn set_dbext<S: Into<Vec<u8>>>(&self, s: S) {
        let s = CString::new(s).unwrap();
        unsafe { alpm_option_set_dbext(self.handle, s.as_ptr()) };
    }

    pub fn set_default_siglevel(&self, s: SigLevel) -> Result<()> {
        let ret = unsafe { alpm_option_set_default_siglevel(self.handle, s.bits() as i32) };
        self.check_ret(ret)
    }

    pub fn default_siglevel(&self) -> SigLevel {
        let ret = unsafe { alpm_option_get_default_siglevel(self.handle) };
        SigLevel::from_bits(ret as u32).unwrap()
    }

    pub fn set_local_file_siglevel(&self, s: SigLevel) -> Result<()> {
        let ret = unsafe { alpm_option_set_local_file_siglevel(self.handle, s.bits() as i32) };
        self.check_ret(ret)
    }

    pub fn local_file_siglevel(&self) -> SigLevel {
        let ret = unsafe { alpm_option_get_local_file_siglevel(self.handle) };
        SigLevel::from_bits(ret as u32).unwrap()
    }

    pub fn set_remote_file_siglevel(&self, s: SigLevel) -> Result<()> {
        let ret = unsafe { alpm_option_set_remote_file_siglevel(self.handle, s.bits() as i32) };
        self.check_ret(ret)
    }

    pub fn remote_file_siglevel(&self) -> SigLevel {
        let ret = unsafe { alpm_option_get_remote_file_siglevel(self.handle) };
        SigLevel::from_bits(ret as u32).unwrap()
    }

    pub fn set_disable_dl_timeout(&self, b: bool) {
        let b = if b { 1 } else { 0 };
        unsafe { alpm_option_set_disable_dl_timeout(self.handle, b) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_getters() {
        let handle = Alpm::new("/", "tests/db/").unwrap();
        assert_eq!(handle.root(), "/");
        assert_eq!(
            handle.dbpath().trim_end_matches('/'),
            PathBuf::from("tests/db/")
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
        );
    }

    #[test]
    fn test_setters() {
        let mut handle = Alpm::new("/", "tests/db/").unwrap();

        handle
            .set_hookdirs(["1", "2", "3"].iter().cloned())
            .unwrap();
        handle.add_hookdir("x").unwrap();
        handle
            .set_hookdirs(["a", "b", "c"].iter().cloned())
            .unwrap();
        handle.add_hookdir("z").unwrap();
        let hooks = handle.hookdirs().iter().collect::<Vec<_>>();
        assert_eq!(hooks, vec!["a/", "b/", "c/", "z/"]);

        assert!(!handle.check_space());
        handle.set_check_space(true);
        assert!(handle.check_space());
        handle.set_check_space(false);
        assert!(!handle.check_space());

        assert_eq!(handle.default_siglevel(), SigLevel::NONE);
        handle
            .set_default_siglevel(SigLevel::PACKAGE | SigLevel::DATABASE)
            .unwrap();
        assert_eq!(
            handle.default_siglevel(),
            SigLevel::PACKAGE | SigLevel::DATABASE
        );

        /*let indeps = vec!["a", "b", "c"].into_iter().map(|s| Depend::new(s)).collect::<Vec<_>>();
        let deps = vec!["a", "b", "c"].into_iter().map(|s| Depend::new(s)).collect::<Vec<_>>();
        handle.set_assume_installed(indeps);

        let ai = handle.assume_installed().collect::<Vec<_>>();
        assert_eq!(deps.into_iter().map(|d| d.to_string()).collect::<Vec<_>>(), ai.into_iter().map(|d| d.to_string()).collect::<Vec<_>>());
        */
    }
}
