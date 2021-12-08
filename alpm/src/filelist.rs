use crate::utils::*;
use crate::Result;

use alpm_sys::*;

use std::cell::UnsafeCell;
use std::ffi::CString;
use std::fmt;
use std::mem;
use std::slice;

#[repr(transparent)]
pub struct File {
    inner: alpm_file_t,
}

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
        self.inner.mode
    }
}

// TODO unsound: needs lifetime on handle
pub struct FileList {
    inner: UnsafeCell<alpm_filelist_t>,
}

impl fmt::Debug for FileList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.files()).finish()
    }
}

impl FileList {
    pub(crate) unsafe fn new(files: alpm_filelist_t) -> FileList {
        FileList {
            inner: UnsafeCell::new(files),
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut alpm_filelist_t {
        self.inner.get()
    }

    pub fn files(&self) -> &[File] {
        let files = unsafe { *self.as_ptr() };
        if files.files.is_null() {
            unsafe { slice::from_raw_parts(mem::align_of::<File>() as *const File, 0) }
        } else {
            unsafe { slice::from_raw_parts(files.files as *const File, files.count) }
        }
    }

    pub fn contains<S: Into<Vec<u8>>>(&self, path: S) -> Result<Option<File>> {
        let path = CString::new(path).unwrap();
        let file = unsafe { alpm_filelist_contains(self.as_ptr(), path.as_ptr()) };

        if file.is_null() {
            Ok(None)
        } else {
            let file = unsafe { *file };
            Ok(Some(File { inner: file }))
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
        assert!(Some(files.files()).is_some());

        let db = handle.localdb();
        let pkg = db.pkg("linux").unwrap();
        let files = pkg.files();

        assert!(!files.files().is_empty());
        assert!(Some(files.files()).is_some());

        let file = files.contains("boot/").unwrap().unwrap();
        assert_eq!(file.name(), "boot/");
        assert!(files.contains("aaaaa/").unwrap().is_none());
    }
}
