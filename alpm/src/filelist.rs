use crate::utils::*;

use alpm_sys::*;

use std::cell::UnsafeCell;
use std::ffi::CString;
use std::fmt;
use std::marker::PhantomData;
use std::slice;

#[repr(transparent)]
pub struct File {
    inner: alpm_file_t,
}

unsafe impl Send for File {}
unsafe impl Sync for File {}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File")
            .field("name", &self.name())
            .field("size", &self.size())
            .field("mode", &self.mode())
            .finish()
    }
}

impl File {
    pub fn name(&self) -> &str {
        unsafe { from_cstr(self.inner.name) }
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

pub struct FileList<'h> {
    inner: UnsafeCell<alpm_filelist_t>,
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
    pub(crate) unsafe fn new<'a>(files: alpm_filelist_t) -> FileList<'a> {
        FileList {
            inner: UnsafeCell::new(files),
            _marker: PhantomData,
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut alpm_filelist_t {
        self.inner.get()
    }

    pub fn files(&self) -> &[File] {
        let files = unsafe { *self.as_ptr() };
        if files.files.is_null() {
            &[]
        } else {
            unsafe { slice::from_raw_parts(files.files as *const File, files.count) }
        }
    }

    pub fn contains<S: Into<Vec<u8>>>(&self, path: S) -> Option<File> {
        let path = CString::new(path).unwrap();
        let file = unsafe { alpm_filelist_contains(self.as_ptr(), path.as_ptr()) };

        if file.is_null() {
            None
        } else {
            let file = unsafe { *file };
            Some(File { inner: file })
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
        assert_eq!(file.name(), "boot/");
        assert!(files.contains("aaaaa/").is_none());
    }
}
