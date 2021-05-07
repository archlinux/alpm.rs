use crate::utils::*;
use crate::{Alpm, AlpmListMut, AsAlpmListItemPtr, AsPkg, Dep, IntoRawAlpmList};

use alpm_sys::alpm_fileconflicttype_t::*;
use alpm_sys::*;

use std::fmt;
use std::marker::PhantomData;
use std::mem::transmute;

pub struct OwnedConflict {
    conflict: Conflict<'static>,
}

impl OwnedConflict {
    pub(crate) unsafe fn from_ptr(ptr: *mut alpm_conflict_t) -> OwnedConflict {
        OwnedConflict {
            conflict: Conflict {
                inner: ptr,
                phantom: PhantomData,
            },
        }
    }
}

impl fmt::Debug for OwnedConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.conflict, f)
    }
}

pub struct Conflict<'a> {
    pub(crate) inner: *mut alpm_conflict_t,
    pub(crate) phantom: PhantomData<&'a ()>,
}

impl<'a> fmt::Debug for Conflict<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Conflict")
            .field("package1", &self.package1())
            .field("package1_hash", &self.package1_hash())
            .field("package2", &self.package2())
            .field("package2_hash", &self.package2_hash())
            .field("reason", &self.reason())
            .finish()
    }
}

impl std::ops::Deref for OwnedConflict {
    type Target = Conflict<'static>;

    fn deref(&self) -> &Self::Target {
        &self.conflict
    }
}

impl Drop for OwnedConflict {
    fn drop(&mut self) {
        unsafe { alpm_conflict_free(self.conflict.inner) }
    }
}

impl<'a> Conflict<'a> {
    pub fn package1_hash(&self) -> u64 {
        #[allow(clippy::useless_conversion)]
        unsafe {
            (*self.inner).package1_hash.into()
        }
    }

    pub fn package2_hash(&self) -> u64 {
        #[allow(clippy::useless_conversion)]
        unsafe {
            (*self.inner).package2_hash.into()
        }
    }

    pub fn package1(&self) -> &'a str {
        unsafe { from_cstr((*self.inner).package1) }
    }

    pub fn package2(&self) -> &'a str {
        unsafe { from_cstr((*self.inner).package2) }
    }

    pub fn reason(&self) -> Dep<'a> {
        unsafe { Dep::from_ptr((*self.inner).reason) }
    }

    pub(crate) unsafe fn from_ptr<'b>(ptr: *mut alpm_conflict_t) -> Conflict<'b> {
        Conflict {
            inner: ptr,
            phantom: PhantomData,
        }
    }
}

#[repr(u32)]
#[derive(Debug)]
pub enum FileConflictType {
    Target = ALPM_FILECONFLICT_TARGET as u32,
    Filesystem = ALPM_FILECONFLICT_FILESYSTEM as u32,
}

pub struct FileConflict<'a> {
    pub(crate) inner: *mut alpm_fileconflict_t,
    pub(crate) phantom: PhantomData<&'a ()>,
}

impl<'a> fmt::Debug for FileConflict<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileConflict")
            .field("target", &self.target())
            .field("conflict_type", &self.conflict_type())
            .field("file", &self.file())
            .field("conflicting_target", &self.conflicting_target())
            .finish()
    }
}

impl std::ops::Deref for OwnedFileConflict {
    type Target = FileConflict<'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct OwnedFileConflict {
    pub(crate) inner: FileConflict<'static>,
}

impl fmt::Debug for OwnedFileConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl<'a> FileConflict<'a> {
    pub fn target(&self) -> &'a str {
        unsafe { from_cstr((*self.inner).target) }
    }

    pub fn conflict_type(&self) -> FileConflictType {
        let t = unsafe { (*self.inner).type_ };
        unsafe { transmute::<alpm_fileconflicttype_t, FileConflictType>(t) }
    }

    pub fn file(&self) -> &'a str {
        unsafe { from_cstr((*self.inner).file) }
    }

    pub fn conflicting_target(&self) -> Option<&'a str> {
        let s = unsafe { from_cstr((*self.inner).ctarget) };

        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}

impl Drop for OwnedFileConflict {
    fn drop(&mut self) {
        unsafe { alpm_fileconflict_free(self.inner.inner) }
    }
}

impl Alpm {
    pub fn check_conflicts<'a, P: 'a + AsPkg + AsAlpmListItemPtr<'a>, L: IntoRawAlpmList<'a, P>>(
        &self,
        list: L,
    ) -> AlpmListMut<OwnedConflict> {
        let list = unsafe { list.into_raw_alpm_list() };
        let ret = unsafe { alpm_checkconflicts(self.handle, list.list()) };
        AlpmListMut::from_parts(self, ret)
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
        assert_eq!(conflict.package1(), "i3-gaps");
        assert_eq!(conflict.package2(), "i3-wm");

        let xterm = handle.syncdbs().find_satisfier("xterm").unwrap();
        let systemd = handle.syncdbs().find_satisfier("systemd").unwrap();
        let conflicts = handle.check_conflicts(vec![xterm, systemd].iter());
        assert!(conflicts.is_empty());
    }
}
