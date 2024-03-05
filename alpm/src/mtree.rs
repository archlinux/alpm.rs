use crate::Pkg;
use alpm_sys::*;

use std::ptr::NonNull;

use libarchive::archive::Handle;
use libarchive::reader::ReaderEntry;
use libarchive3_sys::ffi::*;

use std::{fmt, ptr};

pub struct MTree<'h> {
    pkg: &'h Pkg,
    archive: NonNull<archive>,
}

impl<'h> fmt::Debug for MTree<'h> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MTree").field("pkg", &self.pkg).finish()
    }
}

impl<'h> Handle for MTree<'h> {
    unsafe fn handle(&self) -> *mut Struct_archive {
        self.archive.as_ptr() as *mut Struct_archive
    }
}

impl<'h> Drop for MTree<'h> {
    fn drop(&mut self) {
        unsafe { alpm_pkg_mtree_close(self.pkg.as_ptr(), self.as_ptr()) };
    }
}

impl<'h> Iterator for MTree<'h> {
    type Item = ReaderEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut entry = ptr::null_mut();
        let ret = unsafe { alpm_pkg_mtree_next(self.pkg.as_ptr(), self.as_ptr(), &mut entry) };

        if ret == ARCHIVE_OK {
            Some(ReaderEntry::new(entry as *mut Struct_archive_entry))
        } else {
            None
        }
    }
}

impl<'h> MTree<'h> {
    pub(crate) unsafe fn new<'a>(pkg: &'a Pkg, archive: *mut archive) -> MTree<'a> {
        MTree {
            pkg,
            archive: NonNull::new_unchecked(archive),
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut archive {
        self.archive.as_ptr()
    }
}

#[cfg(test)]
mod tests {
    use crate::Alpm;
    use libarchive::archive::Entry;

    #[test]
    fn test_mtree() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.localdb();
        let pkg = db.pkg("vifm").unwrap();
        let mut mtree = pkg.mtree().unwrap();

        println!("entries:");
        let file = mtree.next().unwrap();
        assert!(file.pathname() == "./.BUILDINFO");
        assert!(file.size() == 4900);
        assert!(mtree.count() > 10);
    }
}
