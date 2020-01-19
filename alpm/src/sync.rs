use crate::{Alpm, AlpmList, Db, FreeMethod, Package, Result};

use std::ffi::CString;

use alpm_sys::*;

impl<'a> Package<'a> {
    pub fn sync_new_version(&self, dbs: AlpmList<Db>) -> Option<Package> {
        let ret = unsafe { alpm_sync_get_new_version(self.pkg, dbs.list) };

        if ret.is_null() {
            None
        } else {
            Some(Package {
                handle: self.handle,
                pkg: ret,
                drop: false,
            })
        }
    }

    pub fn download_size(&self) -> i64 {
        let size = unsafe { alpm_pkg_download_size(self.pkg) };
        size as i64
    }
}

impl Alpm {
    pub fn find_group_pkgs<'a, S: Into<String>>(
        &'a self,
        dbs: AlpmList<Db>,
        s: S,
    ) -> AlpmList<'a, Package<'a>> {
        let name = CString::new(s.into()).unwrap();
        let ret = unsafe { alpm_find_group_pkgs(dbs.list, name.as_ptr()) };
        AlpmList::new(self, ret, FreeMethod::FreeList)
    }
}

impl Alpm {
    pub fn trans_sysupgrade(&self, enable_downgrade: bool) -> Result<()> {
        let enable_downgrade = if enable_downgrade { 1 } else { 0 };
        let ret = unsafe { alpm_sync_sysupgrade(self.handle, enable_downgrade) };

        self.check_ret(ret)
    }
}
