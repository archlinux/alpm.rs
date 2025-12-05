use alpm_sys::*;

use std::ffi::{CStr, CString};
use std::fmt;
use std::marker::PhantomData;
use std::slice;

#[repr(transparent)]
pub struct File<'h> {
    inner: alpm_file_t,
    _marker: PhantomData<&'h ()>,
}

unsafe impl<'h> Send for File<'h> {}
unsafe impl<'h> Sync for File<'h> {}

impl<'h> fmt::Debug for File<'h> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File")
            .field("name", &self.name())
            .field("size", &self.size())
            .field("mode", &self.mode())
            .finish()
    }
}

impl<'h> File<'h> {
    pub fn name(&self) -> &'h [u8] {
        unsafe { CStr::from_ptr(self.inner.name).to_bytes() }
    }

    pub fn size(&self) -> i64 {
        #[allow(clippy::useless_conversion)]
        self.inner.size.into()
    }

    pub fn mode(&self) -> u32 {
        #[allow(clippy::useless_conversion)]
        self.inner.mode.into()
    }
}

#[repr(transparent)]
pub struct FileList<'h> {
    inner: alpm_filelist_t,
    _marker: PhantomData<&'h ()>,
}

// TODO: unsafe cell is only used to get a mut pointer even though
// it's never actually mutated
// Upstream code should ask for a const pointer
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for FileList<'_> {}
unsafe impl Sync for FileList<'_> {}

impl fmt::Debug for FileList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File")
            .field("files", &self.files())
            .finish()
    }
}

impl FileList<'_> {
    pub(crate) unsafe fn new<'a>(files: *mut alpm_filelist_t) -> &'a FileList<'a> {
        unsafe { &*(files as *const FileList) }
    }

    pub(crate) fn as_ptr(&self) -> *mut alpm_filelist_t {
        self as *const _ as *mut _
    }

    pub fn files(&self) -> &'_ [File<'_>] {
        if self.inner.files.is_null() {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.inner.files as *const File, self.inner.count) }
        }
    }

    pub fn contains<S: Into<Vec<u8>>>(&self, path: S) -> Option<&'_ File> {
        let path = CString::new(path).unwrap();
        let file = unsafe { alpm_filelist_contains(self.as_ptr(), path.as_ptr()) };

        if file.is_null() {
            None
        } else {
            unsafe { Some(&*(file as *const File)) }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Alpm, SigLevel};

    #[test]
    fn test_files() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let files = pkg.files();

        assert!(files.files().is_empty());

        let db = handle.localdb();
        let pkg = db.pkg("linux").unwrap();
        let files = pkg.files();

        assert!(!files.files().is_empty());

        let file = files.contains("boot/").unwrap();
        assert_eq!(file.name(), b"boot/");
        assert!(files.contains(b"aaaaa/").is_none());
        assert!(files.contains("aaaaa/").is_none());
    }
}
