use crate::Package;
use alpm_sys::*;

use libarchive::archive::Handle;
use libarchive::reader::ReaderEntry;
use libarchive3_sys::ffi::*;

use std::{fmt, ptr};

pub struct MTree<'a> {
    pub(crate) pkg: &'a Package<'a>,
    pub(crate) archive: *mut archive,
}

impl<'a> fmt::Debug for MTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MTree").field("pkg", &self.pkg).finish()
    }
}

impl<'a> Handle for MTree<'a> {
    unsafe fn handle(&self) -> *mut Struct_archive {
        self.archive as *mut Struct_archive
    }
}

impl<'a> Drop for MTree<'a> {
    fn drop(&mut self) {
        unsafe { alpm_pkg_mtree_close(self.pkg.pkg, self.archive) };
    }
}

impl<'a> Iterator for MTree<'a> {
    type Item = ReaderEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut entry = ptr::null_mut();
        let ret = unsafe { alpm_pkg_mtree_next(self.pkg.pkg, self.archive, &mut entry) };

        if ret == ARCHIVE_OK {
            Some(ReaderEntry::new(entry as *mut Struct_archive_entry))
        } else {
            None
        }
    }
}
