use crate::utils::*;
use crate::{
    Alpm, AlpmList, AlpmListMut, Group, IntoRawAlpmList, Package, Result, SigLevel, Usage,
};

use std::ffi::CString;
use std::fmt;
use std::ops::Deref;

use alpm_sys::*;

#[derive(Copy, Clone)]
pub struct Db<'a> {
    pub(crate) db: *mut alpm_db_t,
    pub(crate) handle: &'a Alpm,
}

impl<'a> fmt::Debug for Db<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Db").field("name", &self.name()).finish()
    }
}

pub struct DbMut<'a> {
    pub(crate) inner: Db<'a>,
}

impl<'a> fmt::Debug for DbMut<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl<'a> Deref for DbMut<'a> {
    type Target = Db<'a>;

    fn deref(&self) -> &Db<'a> {
        &self.inner
    }
}

impl<'a> From<DbMut<'a>> for Db<'a> {
    fn from(db: DbMut<'a>) -> Db<'a> {
        db.inner
    }
}

impl Alpm {
    pub fn register_syncdb<S: Into<Vec<u8>>>(&self, name: S, sig_level: SigLevel) -> Result<Db> {
        let name = CString::new(name).unwrap();

        let db =
            unsafe { alpm_register_syncdb(self.handle, name.as_ptr(), sig_level.bits() as i32) };

        self.check_null(db)?;
        Ok(Db { db, handle: self })
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
        self.check_ret(unsafe { alpm_unregister_all_syncdbs(self.handle) })
    }
}

impl<'a> DbMut<'a> {
    pub fn unregister(self) {
        unsafe { alpm_db_unregister(self.db) };
    }

    pub fn add_server<S: Into<Vec<u8>>>(&self, server: S) -> Result<()> {
        let server = CString::new(server).unwrap();
        let ret = unsafe { alpm_db_add_server(self.db, server.as_ptr()) };
        self.handle.check_ret(ret)
    }

    pub fn set_servers<L: IntoRawAlpmList<'a, String>>(&self, list: L) -> Result<()> {
        let list = unsafe { list.into_raw_alpm_list() };
        let ret = unsafe { alpm_db_set_servers(self.db, alpm_list_strdup(list.list())) };
        self.handle.check_ret(ret)
    }

    pub fn remove_server<S: Into<Vec<u8>>>(&self, server: S) -> Result<()> {
        let server = CString::new(server).unwrap();
        let ret = unsafe { alpm_db_remove_server(self.db, server.as_ptr()) };
        self.handle.check_ret(ret)
    }
}

impl<'a> Db<'a> {
    pub fn name(&self) -> &'a str {
        let name = unsafe { alpm_db_get_name(self.db) };
        unsafe { from_cstr(name) }
    }

    pub fn servers(&self) -> AlpmList<'a, &'a str> {
        let list = unsafe { alpm_db_get_servers(self.db) };
        AlpmList::from_parts(self.handle, list)
    }

    pub fn pkg<S: Into<Vec<u8>>>(&self, name: S) -> Result<Package<'a>> {
        let name = CString::new(name).unwrap();
        let pkg = unsafe { alpm_db_get_pkg(self.db, name.as_ptr()) };
        self.handle.check_null(pkg)?;
        unsafe { Ok(Package::new(self.handle, pkg)) }
    }

    pub fn pkgs(&self) -> AlpmList<'a, Package<'a>> {
        let pkgs = unsafe { alpm_db_get_pkgcache(self.db) };
        AlpmList::from_parts(self.handle, pkgs)
    }

    pub fn group<S: Into<Vec<u8>>>(&self, name: S) -> Result<Group<'a>> {
        let name = CString::new(name).unwrap();
        let group = unsafe { alpm_db_get_group(self.db, name.as_ptr()) };
        self.handle.check_null(group)?;
        Ok(Group {
            handle: self.handle,
            inner: group,
        })
    }

    pub fn set_usage(&self, usage: Usage) -> Result<()> {
        let ret = unsafe { alpm_db_set_usage(self.db, usage.bits() as i32) };
        self.handle.check_ret(ret)
    }

    pub fn search<L>(&self, list: L) -> Result<AlpmListMut<'a, Package<'a>>>
    where
        L: IntoRawAlpmList<'a, String>,
    {
        let mut ret = std::ptr::null_mut();
        let list = unsafe { list.into_raw_alpm_list() };
        let ok = unsafe { alpm_db_search(self.db, list.list(), &mut ret) };
        self.handle.check_ret(ok)?;
        Ok(AlpmListMut::from_parts(self.handle, ret))
    }

    pub fn groups(&self) -> Result<AlpmListMut<'a, Group<'a>>> {
        let groups = unsafe { alpm_db_get_groupcache(self.db) };
        self.handle.check_null(groups)?;
        Ok(AlpmListMut::from_parts(self.handle, groups))
    }

    pub fn siglevel(&self) -> SigLevel {
        let siglevel = unsafe { alpm_db_get_siglevel(self.db) };
        SigLevel::from_bits(siglevel as u32).unwrap()
    }

    pub fn is_valid(&self) -> Result<()> {
        let ret = unsafe { alpm_db_get_valid(self.db) };
        self.handle.check_ret(ret)
    }

    pub fn usage(&self) -> Result<Usage> {
        let mut usage = 0;

        let ret = unsafe { alpm_db_get_usage(self.db, &mut usage) };
        self.handle.check_ret(ret)?;

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
        db.set_servers(servers2.iter()).unwrap();

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

        let mut list: AlpmListMut<String> = AlpmListMut::new(&handle);
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
