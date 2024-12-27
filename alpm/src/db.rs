use crate::utils::*;
use crate::{
    Alpm, AlpmList, AlpmListMut, AsAlpmList, Error, Group, Package, Result, SigLevel, Usage,
};

use std::cell::UnsafeCell;
use std::ffi::CString;
use std::fmt;
use std::ops::Deref;
use std::os::raw::c_int;

use alpm_sys::*;

#[doc(alias("repo", "repository"))]
#[repr(transparent)]
pub struct Db {
    db: UnsafeCell<alpm_db_t>,
}

impl fmt::Debug for Db {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Db").field("name", &self.name()).finish()
    }
}

pub struct DbMut<'h> {
    pub(crate) inner: &'h Db,
}

impl fmt::Debug for DbMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl Deref for DbMut<'_> {
    type Target = Db;

    fn deref(&self) -> &Db {
        self.inner
    }
}

impl Alpm {
    pub fn register_syncdb<S: Into<Vec<u8>>>(&self, name: S, sig_level: SigLevel) -> Result<&Db> {
        let name = CString::new(name).unwrap();

        let db =
            unsafe { alpm_register_syncdb(self.as_ptr(), name.as_ptr(), sig_level.bits() as i32) };

        self.check_null(db)?;
        unsafe { Ok(Db::from_ptr(db)) }
    }

    pub fn register_syncdb_mut<S: Into<Vec<u8>>>(
        &mut self,
        name: S,
        sig_level: SigLevel,
    ) -> Result<DbMut> {
        let db = self.register_syncdb(name, sig_level)?;
        Ok(DbMut { inner: db })
    }

    pub fn unregister_all_syncdbs(&mut self) -> Result<()> {
        self.check_ret(unsafe { alpm_unregister_all_syncdbs(self.as_ptr()) })
    }
}

impl DbMut<'_> {
    pub(crate) unsafe fn from_ptr<'a>(db: *mut alpm_db_t) -> DbMut<'a> {
        DbMut {
            inner: Db::from_ptr(db),
        }
    }

    pub fn unregister(self) {
        unsafe { alpm_db_unregister(self.as_ptr()) };
    }

    pub fn add_server<S: Into<Vec<u8>>>(&self, server: S) -> Result<()> {
        let server = CString::new(server).unwrap();
        let ret = unsafe { alpm_db_add_server(self.as_ptr(), server.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn set_servers<'a, L: AsAlpmList<&'a str>>(&self, list: L) -> Result<()> {
        list.with(|list| {
            let ret = unsafe { alpm_db_set_servers(self.as_ptr(), list.as_ptr()) };
            self.check_ret(ret)
        })
    }

    pub fn remove_server<S: Into<Vec<u8>>>(&self, server: S) -> Result<()> {
        let server = CString::new(server).unwrap();
        let ret = unsafe { alpm_db_remove_server(self.as_ptr(), server.as_ptr()) };
        self.check_ret(ret)
    }
}

impl Db {
    pub(crate) unsafe fn from_ptr<'a>(db: *mut alpm_db_t) -> &'a Db {
        &*(db as *mut Db)
    }

    pub(crate) fn handle_ptr(&self) -> *mut alpm_handle_t {
        unsafe { alpm_db_get_handle(self.as_ptr()) }
    }

    pub(crate) fn last_error(&self) -> Error {
        unsafe { Error::new(alpm_errno(alpm_db_get_handle(self.as_ptr()))) }
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

    pub fn as_ptr(&self) -> *mut alpm_db_t {
        self.db.get()
    }

    pub fn name(&self) -> &str {
        let name = unsafe { alpm_db_get_name(self.as_ptr()) };
        unsafe { from_cstr(name) }
    }

    pub fn servers(&self) -> AlpmList<&str> {
        let list = unsafe { alpm_db_get_servers(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn pkg<S: Into<Vec<u8>>>(&self, name: S) -> Result<&Package> {
        let name = CString::new(name).unwrap();
        let pkg = unsafe { alpm_db_get_pkg(self.as_ptr(), name.as_ptr()) };
        self.check_null(pkg)?;
        unsafe { Ok(Package::from_ptr(pkg)) }
    }

    #[doc(alias = "pkgcache")]
    pub fn pkgs(&self) -> AlpmList<&Package> {
        let pkgs = unsafe { alpm_db_get_pkgcache(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(pkgs) }
    }

    pub fn group<S: Into<Vec<u8>>>(&self, name: S) -> Result<&Group> {
        let name = CString::new(name).unwrap();
        let group = unsafe { alpm_db_get_group(self.as_ptr(), name.as_ptr()) };
        self.check_null(group)?;
        unsafe { Ok(Group::from_ptr(group)) }
    }

    pub fn set_usage(&self, usage: Usage) -> Result<()> {
        let ret = unsafe { alpm_db_set_usage(self.as_ptr(), usage.bits() as i32) };
        self.check_ret(ret)
    }

    pub fn search<'a, L>(&'a self, list: L) -> Result<AlpmListMut<&'a Package>>
    where
        L: AsAlpmList<&'a str>,
    {
        list.with(|list| {
            let mut ret = std::ptr::null_mut();
            let ok = unsafe { alpm_db_search(self.as_ptr(), list.as_ptr(), &mut ret) };
            self.check_ret(ok)?;
            unsafe { Ok(AlpmListMut::from_ptr(ret)) }
        })
    }

    #[doc(alias = "groupcache")]
    pub fn groups(&self) -> Result<AlpmList<&Group>> {
        let groups = unsafe { alpm_db_get_groupcache(self.as_ptr()) };
        self.check_null(groups)?;
        unsafe { Ok(AlpmList::from_ptr(groups)) }
    }

    pub fn siglevel(&self) -> SigLevel {
        let siglevel = unsafe { alpm_db_get_siglevel(self.as_ptr()) };
        SigLevel::from_bits(siglevel as u32).unwrap()
    }

    pub fn is_valid(&self) -> Result<()> {
        let ret = unsafe { alpm_db_get_valid(self.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn usage(&self) -> Result<Usage> {
        let mut usage = 0;

        let ret = unsafe { alpm_db_get_usage(self.as_ptr(), &mut usage) };
        self.check_ret(ret)?;

        let usage = Usage::from_bits(usage as u32).unwrap();
        Ok(usage)
    }
}

#[cfg(test)]
mod tests {
    use crate::SigLevel;
    use crate::{Alpm, AlpmListMut};

    #[test]
    fn test_register() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("foo", SigLevel::NONE).unwrap();

        assert_eq!(db.name(), "foo");
    }

    #[test]
    fn test_servers() {
        let mut handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb_mut("foo", SigLevel::NONE).unwrap();
        assert_eq!(db.name(), "foo");
        let servers = vec!["a", "bb", "ccc"];

        for server in &servers {
            db.add_server(*server).unwrap();
        }

        let servers2 = db
            .servers()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        db.set_servers(servers2.iter()).unwrap();
        let servers2 = db
            .servers()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        db.set_servers(servers2.into_iter()).unwrap();

        assert_eq!(servers, db.servers().iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_set_servers() {
        let mut handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb_mut("foo", SigLevel::NONE).unwrap();
        assert_eq!(db.name(), "foo");
        let servers = vec!["a", "bb", "ccc"];

        db.set_servers(servers.iter().cloned()).unwrap();

        assert_eq!(servers, db.servers().iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_mut() {
        let mut handle = Alpm::new("/", "tests/db").unwrap();
        handle.register_syncdb_mut("foo", SigLevel::NONE).unwrap();
        handle.register_syncdb_mut("bar", SigLevel::NONE).unwrap();

        for db in handle.syncdbs_mut() {
            db.add_server("foo").unwrap();
        }

        for db in handle.syncdbs_mut() {
            db.add_server("bar").unwrap();
        }

        for db in handle.syncdbs() {
            assert_eq!(db.servers().iter().collect::<Vec<_>>(), vec!["foo", "bar"]);
        }

        for db in handle.syncdbs_mut() {
            db.unregister();
        }

        assert!(handle.syncdbs().is_empty());
    }

    #[test]
    fn test_pkg() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        assert!(pkg.version().as_str() == "5.1.8.arch1-1");
    }

    #[test]
    fn test_search() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let res = db
            .search(["^mkinitcpio-nfs-utils$"].iter().cloned())
            .unwrap();
        let res = res.iter().collect::<Vec<_>>();

        for _ in &res {}
        for _ in &res {}

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name(), "mkinitcpio-nfs-utils");

        let mut list: AlpmListMut<String> = AlpmListMut::new();
        list.push("pacman".to_string());

        let pkgs = db.search(&list).unwrap();
        assert!(!pkgs.is_empty());

        db.search(["pacman"].iter().cloned()).unwrap();
        db.search(vec!["pacman".to_string()].into_iter()).unwrap();
    }

    #[test]
    fn test_group() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let base = db.group("base").unwrap();
        assert_eq!(base.name(), "base");
        assert!(base.packages().len() > 10);
        assert!(base.packages().len() < 100);
    }
}
