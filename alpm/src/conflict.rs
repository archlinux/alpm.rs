use crate::{utils::*, Package};
use crate::{Alpm, AlpmListMut, AsAlpmList, Dep, Pkg};

use alpm_sys::alpm_fileconflicttype_t::*;
use alpm_sys::*;

use std::fmt;
use std::mem::transmute;
use std::ptr::NonNull;

pub struct OwnedConflict {
    inner: NonNull<alpm_conflict_t>,
}

unsafe impl Send for OwnedConflict {}
unsafe impl Sync for OwnedConflict {}

impl OwnedConflict {
    pub(crate) unsafe fn from_ptr(ptr: *mut alpm_conflict_t) -> OwnedConflict {
        OwnedConflict {
            inner: NonNull::new_unchecked(ptr),
        }
    }
}

impl AsRef<Conflict> for OwnedConflict {
    fn as_ref(&self) -> &Conflict {
        self
    }
}

impl fmt::Debug for OwnedConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Conflict::fmt(self, f)
    }
}

#[repr(transparent)]
pub struct Conflict {
    inner: alpm_conflict_t,
}

unsafe impl Send for Conflict {}
unsafe impl Sync for Conflict {}

impl fmt::Debug for Conflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "git"))]
        {
            f.debug_struct("Conflict")
                .field("package1", &self.package1())
                .field("package2", &self.package2())
                .field("reason", &self.reason())
                .finish()
        }
        // Implement properly when we merge the no handle code
        #[cfg(feature = "git")]
        {
            f.debug_struct("Conflict").finish()
        }
    }
}

impl AsRef<Conflict> for Conflict {
    fn as_ref(&self) -> &Conflict {
        self
    }
}

impl std::ops::Deref for OwnedConflict {
    type Target = Conflict;

    fn deref(&self) -> &Self::Target {
        unsafe { Conflict::from_ptr(self.inner.as_ptr()) }
    }
}

impl Drop for OwnedConflict {
    fn drop(&mut self) {
        unsafe { alpm_conflict_free(self.inner.as_ptr()) }
    }
}

impl Conflict {
    pub(crate) unsafe fn from_ptr<'a>(ptr: *mut alpm_conflict_t) -> &'a Conflict {
        &*(ptr as *mut Conflict)
    }

    pub(crate) fn as_ptr(&self) -> *const alpm_conflict_t {
        &self.inner
    }

    pub fn package1(&self) -> &Package {
        unsafe { Package::from_ptr((*self.as_ptr()).package1) }
    }

    pub fn package2(&self) -> &Package {
        unsafe { Package::from_ptr((*self.as_ptr()).package2) }
    }

    pub fn reason(&self) -> &Dep {
        unsafe { Dep::from_ptr((*self.as_ptr()).reason) }
    }
}

#[repr(u32)]
#[derive(Debug)]
pub enum FileConflictType {
    Target = ALPM_FILECONFLICT_TARGET as u32,
    Filesystem = ALPM_FILECONFLICT_FILESYSTEM as u32,
}

#[repr(transparent)]
pub struct FileConflict {
    inner: alpm_fileconflict_t,
}

unsafe impl Sync for FileConflict {}
unsafe impl Send for FileConflict {}

impl fmt::Debug for FileConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileConflict")
            .field("target", &self.target())
            .field("conflict_type", &self.conflict_type())
            .field("file", &self.file())
            .field("conflicting_target", &self.conflicting_target())
            .finish()
    }
}

impl AsRef<FileConflict> for FileConflict {
    fn as_ref(&self) -> &FileConflict {
        self
    }
}

impl std::ops::Deref for OwnedFileConflict {
    type Target = FileConflict;

    fn deref(&self) -> &Self::Target {
        unsafe { FileConflict::from_ptr(self.inner.as_ptr()) }
    }
}

pub struct OwnedFileConflict {
    pub(crate) inner: NonNull<alpm_fileconflict_t>,
}

impl AsRef<FileConflict> for OwnedFileConflict {
    fn as_ref(&self) -> &FileConflict {
        self
    }
}

unsafe impl Sync for OwnedFileConflict {}
unsafe impl Send for OwnedFileConflict {}

impl OwnedFileConflict {
    pub(crate) unsafe fn from_ptr(ptr: *mut alpm_fileconflict_t) -> OwnedFileConflict {
        OwnedFileConflict {
            inner: NonNull::new_unchecked(ptr),
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut alpm_fileconflict_t {
        self.inner.as_ptr()
    }
}

impl fmt::Debug for OwnedFileConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl FileConflict {
    pub(crate) unsafe fn from_ptr<'a>(ptr: *mut alpm_fileconflict_t) -> &'a FileConflict {
        &*(ptr as *mut FileConflict)
    }

    pub(crate) fn as_ptr(&self) -> *const alpm_fileconflict_t {
        &self.inner
    }

    pub fn target(&self) -> &str {
        unsafe { from_cstr((*self.as_ptr()).target) }
    }

    pub fn conflict_type(&self) -> FileConflictType {
        let t = unsafe { (*self.as_ptr()).type_ };
        unsafe { transmute::<alpm_fileconflicttype_t, FileConflictType>(t) }
    }

    pub fn file(&self) -> &str {
        unsafe { from_cstr((*self.as_ptr()).file) }
    }

    // TODO: target is "" when empty. should be null instead.
    pub fn conflicting_target(&self) -> Option<&str> {
        let s = unsafe { from_cstr((*self.as_ptr()).target) };

        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}

impl Drop for OwnedFileConflict {
    fn drop(&mut self) {
        unsafe { alpm_fileconflict_free(self.as_ptr()) }
    }
}

impl Alpm {
    pub fn check_conflicts<'a, L: AsAlpmList<&'a Pkg>>(
        &self,
        list: L,
    ) -> AlpmListMut<OwnedConflict> {
        list.with(|list| {
            let ret = unsafe { alpm_checkconflicts(self.as_ptr(), list.as_ptr()) };
            unsafe { AlpmListMut::from_ptr(ret) }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SigLevel;

    #[test]
    fn test_check_conflicts() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        handle.register_syncdb("core", SigLevel::NONE).unwrap();
        handle.register_syncdb("extra", SigLevel::NONE).unwrap();
        handle.register_syncdb("community", SigLevel::NONE).unwrap();

        let i3 = handle.syncdbs().find_satisfier("i3-wm").unwrap();
        let i3gaps = handle.syncdbs().find_satisfier("i3-gaps").unwrap();
        let conflicts = handle.check_conflicts(vec![i3, i3gaps].iter());
        let conflict = conflicts.first().unwrap();
        assert_eq!(conflict.package1().name(), "i3-gaps");
        assert_eq!(conflict.package2().name(), "i3-wm");

        let xterm = handle.syncdbs().find_satisfier("xterm").unwrap();
        let systemd = handle.syncdbs().find_satisfier("systemd").unwrap();
        let conflicts = handle.check_conflicts(vec![xterm, systemd].iter());
        assert!(conflicts.is_empty());
    }
}
