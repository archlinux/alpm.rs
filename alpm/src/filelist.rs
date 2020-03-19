use crate::utils::*;
use crate::Result;

use alpm_sys::*;

use std::ffi::CString;
use std::slice;

#[derive(Debug)]
pub struct File {
    inner: alpm_file_t,
}

impl File {
    pub fn name(&self) -> &str {
        unsafe { from_cstr(self.inner.name) }
    }

    pub fn size(&self) -> i64 {
        #[allow(clippy::identity_conversion)]
        self.inner.size.into()
    }

    pub fn mode(&self) -> u32 {
        self.inner.mode
    }
}

#[derive(Debug)]
pub struct FileList {
    pub(crate) inner: alpm_filelist_t,
}

impl FileList {
    pub fn files(&self) -> &[File] {
        unsafe { slice::from_raw_parts(self.inner.files as *const File, self.inner.count) }
    }

    pub fn contains(&self, path: impl AsRef<str>) -> Result<Option<File>> {
        let path = CString::new(path.as_ref()).unwrap();
        let file = unsafe {
            alpm_filelist_contains(
                &self.inner as *const alpm_filelist_t as *mut alpm_filelist_t,
                path.as_ptr(),
            )
        };

        if file.is_null() {
            Ok(None)
        } else {
            let file = unsafe { *file };
            Ok(Some(File { inner: file }))
        }
    }
}
