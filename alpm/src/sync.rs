use crate::{Alpm, AlpmList, AlpmListMut, AsRawAlpmList, Db, Package, Result};

use std::ffi::CString;

use alpm_sys::*;

impl<'a> Package<'a> {
    pub fn sync_new_version<T: AsRawAlpmList<'a, Db<'a>>>(&self, dbs: T) -> Option<Package> {
        let dbs = unsafe { dbs.as_raw_alpm_list() };
        let ret = unsafe { alpm_sync_get_new_version(self.pkg, dbs.list()) };

        if ret.is_null() {
            None
        } else {
            unsafe { Some(Package::new(self.handle, ret)) }
        }
    }

    pub fn download_size(&self) -> i64 {
        let size = unsafe { alpm_pkg_download_size(self.pkg) };
        size as i64
    }
}

impl Alpm {
    pub fn find_group_pkgs<'a, S: Into<Vec<u8>>>(
        &'a self,
        dbs: AlpmList<Db>,
        s: S,
    ) -> AlpmListMut<'a, Package<'a>> {
        let name = CString::new(s).unwrap();
        let ret = unsafe { alpm_find_group_pkgs(dbs.list, name.as_ptr()) };
        AlpmListMut::from_parts(self, ret)
    }
}

impl Alpm {
    pub fn sync_sysupgrade(&self, enable_downgrade: bool) -> Result<()> {
        let ret = unsafe { alpm_sync_sysupgrade(self.handle, enable_downgrade as _) };
        self.check_ret(ret)
    }
}
