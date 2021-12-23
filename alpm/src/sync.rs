use crate::{Alpm, AlpmList, AlpmListMut, AsAlpmList, Db, Package, Result};

use std::ffi::CString;

use alpm_sys::*;

impl Package {
    pub fn sync_new_version<'a, T: AsAlpmList<&'a Db>>(&self, dbs: T) -> Option<&'a Package> {
        dbs.with(|dbs| {
            let ret = unsafe { alpm_sync_get_new_version(self.as_ptr(), dbs.as_ptr()) };

            if ret.is_null() {
                None
            } else {
                unsafe { Some(Package::from_ptr(ret)) }
            }
        })
    }

    pub fn download_size(&self) -> i64 {
        let size = unsafe { alpm_pkg_download_size(self.as_ptr()) };
        size as i64
    }
}

impl Alpm {
    pub fn find_group_pkgs<'a, S: Into<Vec<u8>>>(
        &'a self,
        dbs: AlpmList<&Db>,
        s: S,
    ) -> AlpmListMut<&'a Package> {
        let name = CString::new(s).unwrap();
        let ret = unsafe { alpm_find_group_pkgs(dbs.as_ptr(), name.as_ptr()) };
        unsafe { AlpmListMut::from_ptr(ret) }
    }
}

impl Alpm {
    pub fn sync_sysupgrade(&self, enable_downgrade: bool) -> Result<()> {
        let ret = unsafe { alpm_sync_sysupgrade(self.as_ptr(), enable_downgrade as _) };
        self.check_ret(ret)
    }
}
