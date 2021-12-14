use crate::{Alpm, AlpmList, AlpmListMut, Db, IntoRawAlpmList, Package, Result};

use std::ffi::CString;

use alpm_sys::*;

impl<'a> Package<'a> {
    pub fn sync_new_version<T: IntoRawAlpmList<'a, Db<'a>>>(&self, dbs: T) -> Option<Package> {
        let dbs = unsafe { dbs.into_raw_alpm_list() };
        let ret = unsafe { alpm_sync_get_new_version(self.pkg.as_ptr(), dbs.list()) };

        if ret.is_null() {
            None
        } else {
            unsafe { Some(Package::from_ptr(ret)) }
        }
    }

    pub fn download_size(&self) -> i64 {
        let size = unsafe { alpm_pkg_download_size(self.pkg.as_ptr()) };
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
        unsafe { AlpmListMut::from_ptr(ret) }
    }
}

impl Alpm {
    pub fn sync_sysupgrade(&self, enable_downgrade: bool) -> Result<()> {
        let ret = unsafe { alpm_sync_sysupgrade(self.as_ptr(), enable_downgrade as _) };
        self.check_ret(ret)
    }
}
