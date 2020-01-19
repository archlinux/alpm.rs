use crate::utils::*;
use crate::{free, Alpm, AlpmList, FreeMethod, Group, Package, Result, SigLevel, Usage};

use std::ffi::CString;
use std::ops::Deref;

use alpm_sys::*;

#[derive(Debug)]
pub struct Db<'a> {
    pub(crate) db: *mut alpm_db_t,
    pub(crate) handle: &'a Alpm,
}

#[derive(Debug)]
pub struct DbMut<'a> {
    pub(crate) inner: Db<'a>,
}

impl<'a> Deref for DbMut<'a> {
    type Target = Db<'a>;

    fn deref(&self) -> &Db<'a> {
        &self.inner
    }
}

impl<'a> Into<Db<'a>> for DbMut<'a> {
    fn into(self) -> Db<'a> {
        self.inner
    }
}

#[derive(Debug)]
pub struct DbBuilder {
    handle: Alpm,
    pub(crate) db: *mut alpm_db_t,
}

impl Alpm {
    pub fn register_syncdb<S: Into<String>>(&self, name: S, sig_level: SigLevel) -> Result<Db> {
        let name = CString::new(name.into()).unwrap();

        let db =
            unsafe { alpm_register_syncdb(self.handle, name.as_ptr(), sig_level.bits() as i32) };

        self.check_null(db)?;
        Ok(Db { db, handle: self })
    }

    pub fn register_syncdb_mut<S: Into<String>>(
        &mut self,
        name: S,
        sig_level: SigLevel,
    ) -> Result<DbMut> {
        let name = CString::new(name.into()).unwrap();

        let db =
            unsafe { alpm_register_syncdb(self.handle, name.as_ptr(), sig_level.bits() as i32) };

        self.check_null(db)?;
        Ok(DbMut {
            inner: Db { db, handle: self },
        })
    }

    pub fn unregister_all_syncdbs(&mut self) -> Result<()> {
        self.check_ret(unsafe { alpm_unregister_all_syncdbs(self.handle) })
    }
}

impl<'a> DbMut<'a> {
    pub fn unregister(self) {
        unsafe { alpm_db_unregister(self.db) };
    }

    pub fn add_server<S: Into<String>>(&self, server: S) -> Result<()> {
        let server = CString::new(server.into()).unwrap();
        let ret = unsafe { alpm_db_add_server(self.db, server.as_ptr()) };
        self.handle.check_ret(ret)
    }

    pub fn set_servers<S: Into<String>, I: IntoIterator<Item = S>>(&self, list: I) -> Result<()> {
        let list = to_strlist(list);
        let ret = unsafe { alpm_db_set_servers(self.db, list) };
        self.handle.check_ret(ret)
    }

    pub fn remove_server<S: Into<String>>(&self, server: S) -> Result<()> {
        let server = CString::new(server.into()).unwrap();
        let ret = unsafe { alpm_db_remove_server(self.db, server.as_ptr()) };
        self.handle.check_ret(ret)
    }
}

impl<'a> Db<'a> {
    pub fn name(&self) -> &str {
        let name = unsafe { alpm_db_get_name(self.db) };
        unsafe { from_cstr(name) }
    }

    pub fn servers(&self) -> AlpmList<&str> {
        let list = unsafe { alpm_db_get_servers(self.db) };
        AlpmList::new(self.handle, list, FreeMethod::None)
    }

    pub fn pkg<S: Into<String>>(&self, name: S) -> Result<Package<'a>> {
        let name = CString::new(name.into()).unwrap();
        let pkg = unsafe { alpm_db_get_pkg(self.db, name.as_ptr()) };
        self.handle.check_null(pkg)?;
        Ok(Package {
            handle: self.handle,
            pkg,
            drop: false,
        })
    }

    pub fn pkgs(&self) -> Result<AlpmList<'a, Package<'a>>> {
        let pkgs = unsafe { alpm_db_get_pkgcache(self.db) };
        self.handle.check_null(pkgs)?;
        Ok(AlpmList::new(self.handle, pkgs, FreeMethod::None))
    }

    pub fn group<S: Into<String>>(&self, name: S) -> Result<Group> {
        let name = CString::new(name.into()).unwrap();
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

    #[cfg(not(feature = "git"))]
    pub fn search<S: Into<String>, I: IntoIterator<Item = S>>(
        &self,
        list: I,
    ) -> Result<AlpmList<'a, Package<'a>>> {
        let list = to_strlist(list.into_iter());
        let pkgs = unsafe { alpm_db_search(self.db, list) };
        unsafe { alpm_list_free_inner(list, Some(free)) };
        unsafe { alpm_list_free(list) };
        self.handle.check_null(pkgs)?;
        Ok(AlpmList::new(self.handle, pkgs, FreeMethod::FreeList))
    }

    #[cfg(feature = "git")]
    pub fn search<S: Into<String>, I: IntoIterator<Item = S>>(
        &self,
        list: I,
    ) -> Result<AlpmList<'a, Package<'a>>> {
        let list = to_strlist(list.into_iter());
        let mut ret = std::ptr::null_mut();
        let ok = unsafe { alpm_db_search(self.db, list, &mut ret) };
        unsafe { alpm_list_free_inner(list, Some(free)) };
        unsafe { alpm_list_free(list) };
        self.handle.check_ret(ok)?;
        Ok(AlpmList::new(self.handle, ret, FreeMethod::FreeList))
    }

    pub fn groups(&self) -> Result<AlpmList<'a, Group>> {
        let groups = unsafe { alpm_db_get_pkgcache(self.db) };
        self.handle.check_null(groups)?;
        Ok(AlpmList::new(self.handle, groups, FreeMethod::FreeList))
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
    use crate::Alpm;
    use crate::SigLevel;

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

        let servers2 = db.servers().map(|s| s.to_string()).collect::<Vec<_>>();
        db.set_servers(servers2).unwrap();
        let servers2 = db.servers().map(|s| s.to_string()).collect::<Vec<_>>();
        db.set_servers(servers2).unwrap();
        let servers2 = db.servers().map(|s| s.to_string()).collect::<Vec<_>>();
        db.set_servers(servers2).unwrap();
        let servers2 = db.servers().map(|s| s.to_string()).collect::<Vec<_>>();
        db.set_servers(servers2).unwrap();
        let servers2 = db.servers().map(|s| s.to_string()).collect::<Vec<_>>();
        db.set_servers(servers2).unwrap();
        let servers2 = db.servers().map(|s| s.to_string()).collect::<Vec<_>>();
        db.set_servers(servers2).unwrap();
        let servers2 = db.servers().map(|s| s.to_string()).collect::<Vec<_>>();
        db.set_servers(servers2).unwrap();
        let servers2 = db.servers().map(|s| s.to_string()).collect::<Vec<_>>();
        db.set_servers(servers2).unwrap();
        let servers2 = db.servers().map(|s| s.to_string()).collect::<Vec<_>>();
        db.set_servers(servers2).unwrap();
        let servers2 = db.servers().map(|s| s.to_string()).collect::<Vec<_>>();
        db.set_servers(servers2).unwrap();
        let servers2 = db.servers().map(|s| s.to_string()).collect::<Vec<_>>();
        db.set_servers(servers2).unwrap();
        let servers2 = db.servers().map(|s| s.to_string()).collect::<Vec<_>>();
        db.set_servers(servers2).unwrap();

        assert_eq!(servers, db.servers().collect::<Vec<_>>());
    }

    #[test]
    fn test_set_servers() {
        let mut handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb_mut("foo", SigLevel::NONE).unwrap();
        assert_eq!(db.name(), "foo");
        let servers = vec!["a", "bb", "ccc"];

        db.set_servers(servers.iter().cloned()).unwrap();

        assert_eq!(servers, db.servers().collect::<Vec<_>>());
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
            assert_eq!(db.servers().collect::<Vec<_>>(), vec!["foo", "bar"]);
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
        assert!(pkg.version() == "5.1.8.arch1-1");
    }

    #[test]
    fn test_search() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let res = db
            .search(["^mkinitcpio-nfs-utils$"].iter().cloned())
            .unwrap();
        let res = res.collect::<Vec<_>>();

        for _ in &res {}
        for _ in &res {}

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name(), "mkinitcpio-nfs-utils");

        db.search(["["].iter().cloned()).unwrap_err();
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
